//! Accès aux releases GitHub du launcher.
//!
//! Les pages ne parlent jamais à GitHub : elles lisent un cache en mémoire, rafraîchi en arrière-plan
//! (`rafraichir_en_boucle`). Ainsi une panne ou une lenteur de GitHub (502, 504, coupure réseau) ne
//! ralentit ni ne bloque le site : on sert la dernière liste connue.
//!
//! - délai maximal par requête, et une seule requête à la fois ;
//! - requêtes conditionnelles (ETag) : « rien n'a changé » ne coûte rien au quota de l'API ;
//! - après un échec, attente croissante avant de réessayer (30 s, 1 min, 2 min… jusqu'à 10 min) ;
//! - la dernière liste est gardée sur le disque : après un redémarrage, le site a ses données même si
//!   GitHub ne répond pas ;
//! - une ligne de journal quand GitHub tombe, une quand il revient (au lieu d'une par visite).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const USER_AGENT: &str = "misty-site";
/// Rafraîchissement normal.
const RAFRAICHISSEMENT: Duration = Duration::from_secs(300);
/// Attente maximale après des échecs répétés.
const ATTENTE_MAX: Duration = Duration::from_secs(600);
/// Délai maximal d'une requête vers GitHub.
const DELAI_REQUETE: Duration = Duration::from_secs(8);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: Option<String>,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    pub content_type: Option<String>,
}

/// Les deux fichiers publiés à chaque version.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// `Misty-Setup-<version>.exe`
    Setup,
    /// `Misty-<version>.zip`
    Portable,
}

impl Kind {
    pub fn from_slug(s: &str) -> Option<Kind> {
        match s {
            "setup" => Some(Kind::Setup),
            "portable" => Some(Kind::Portable),
            _ => None,
        }
    }

    pub fn matches(self, file: &str) -> bool {
        let f = file.to_ascii_lowercase();
        match self {
            Kind::Setup => f.starts_with("misty-setup-") && f.ends_with(".exe"),
            Kind::Portable => {
                f.starts_with("misty-") && !f.starts_with("misty-setup") && f.ends_with(".zip")
            }
        }
    }
}

impl Release {
    pub fn asset(&self, kind: Kind) -> Option<&Asset> {
        self.assets.iter().find(|a| kind.matches(&a.name))
    }

    pub fn title(&self) -> &str {
        self.name
            .as_deref()
            .filter(|n| !n.trim().is_empty())
            .unwrap_or(&self.tag_name)
    }

    pub fn notes(&self) -> &str {
        self.body.as_deref().unwrap_or("").trim()
    }
}

#[derive(Default)]
struct Etat {
    liste: Option<Arc<Vec<Release>>>,
    etag: Option<String>,
    /// Échecs consécutifs (0 = la dernière tentative a réussi, ou aucune tentative encore)
    echecs: u32,
}

pub struct GitHub {
    http: reqwest::Client,
    api_base: String,
    repo: String,
    token: Option<String>,
    etat: Mutex<Etat>,
    /// Une seule requête vers GitHub à la fois
    requete: tokio::sync::Mutex<()>,
    fichier_cache: PathBuf,
}

enum Reponse {
    Nouvelle(Vec<Release>, Option<String>),
    Inchangee,
}

impl GitHub {
    pub fn new(api_base: String, repo: String, token: Option<String>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(5))
            .timeout(DELAI_REQUETE)
            .build()
            .expect("client HTTP");
        let fichier_cache = std::env::var_os("MISTY_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("misty-site-releases.json");
        let mut etat = Etat::default();
        // Dernière liste connue (redémarrage) : servie tout de suite, rafraîchie ensuite
        if let Some(liste) = std::fs::read(&fichier_cache)
            .ok()
            .and_then(|o| serde_json::from_slice::<Vec<Release>>(&o).ok())
            .filter(|l| !l.is_empty())
        {
            etat.liste = Some(Arc::new(liste));
        }
        Self {
            http,
            api_base,
            repo,
            token,
            etat: Mutex::new(etat),
            requete: tokio::sync::Mutex::new(()),
            fichier_cache,
        }
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Toutes les releases publiées (sans les brouillons), les plus récentes d'abord.
    /// Ne contacte GitHub que si on n'a encore aucune liste et qu'aucun échec n'est en cours
    /// (premier démarrage), et jamais plus longtemps que le délai d'une requête.
    pub async fn releases(&self) -> Option<Arc<Vec<Release>>> {
        let (liste, echecs) = {
            let e = self.etat.lock().unwrap();
            (e.liste.clone(), e.echecs)
        };
        if liste.is_some() || echecs > 0 {
            return liste;
        }
        self.rafraichir().await;
        self.etat.lock().unwrap().liste.clone()
    }

    /// Tâche de fond : rafraîchit la liste toutes les 5 minutes, et attend plus longtemps après des
    /// échecs (sans jamais bloquer les pages).
    pub async fn rafraichir_en_boucle(self: Arc<Self>) {
        loop {
            self.rafraichir().await;
            let echecs = self.etat.lock().unwrap().echecs;
            let attente = if echecs == 0 {
                RAFRAICHISSEMENT
            } else {
                (Duration::from_secs(30) * 2u32.pow((echecs - 1).min(6))).min(ATTENTE_MAX)
            };
            tokio::time::sleep(attente).await;
        }
    }

    async fn rafraichir(&self) {
        let Ok(_verrou) = self.requete.try_lock() else {
            // déjà en cours : on attend simplement son résultat
            let _attente = self.requete.lock().await;
            return;
        };
        let etag = self.etat.lock().unwrap().etag.clone();
        let resultat = self.fetch(etag.as_deref()).await;
        let mut e = self.etat.lock().unwrap();
        match resultat {
            Ok(reponse) => {
                if e.echecs > 0 {
                    eprintln!("GitHub de nouveau joignable (après {} échec(s))", e.echecs);
                }
                e.echecs = 0;
                if let Reponse::Nouvelle(liste, etag) = reponse {
                    if let Ok(json) = serde_json::to_vec(&liste) {
                        let _ = std::fs::write(&self.fichier_cache, json);
                    }
                    e.liste = Some(Arc::new(liste));
                    e.etag = etag;
                }
            }
            Err(err) => {
                e.echecs += 1;
                // une ligne au premier échec, puis une toutes les 10 tentatives
                if e.echecs == 1 || e.echecs % 10 == 0 {
                    let servie = if e.liste.is_some() {
                        "on sert la dernière liste connue"
                    } else {
                        "aucune liste en cache"
                    };
                    eprintln!(
                        "GitHub indisponible ({} échec(s) d'affilée, {servie}) : {err}",
                        e.echecs
                    );
                }
            }
        }
    }

    async fn fetch(&self, etag: Option<&str>) -> Result<Reponse, reqwest::Error> {
        let url = format!(
            "{}/repos/{}/releases?per_page=100",
            self.api_base, self.repo
        );
        let mut req = self
            .http
            .get(url)
            .header("Accept", "application/vnd.github+json");
        if let Some(t) = &self.token {
            req = req.bearer_auth(t);
        }
        if let Some(etag) = etag {
            req = req.header("If-None-Match", etag);
        }
        let reponse = req.send().await?;
        if reponse.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(Reponse::Inchangee);
        }
        let reponse = reponse.error_for_status()?;
        let etag = reponse
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let mut liste: Vec<Release> = reponse.json().await?;
        liste.retain(|r| !r.draft);
        Ok(Reponse::Nouvelle(liste, etag))
    }
}

/// La version mise en avant : la plus récente stable, sinon la plus récente tout court.
pub fn latest(list: &[Release]) -> Option<&Release> {
    list.iter().find(|r| !r.prerelease).or_else(|| list.first())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn asset_names() {
        assert!(Kind::Setup.matches("Misty-Setup-0.3.1.exe"));
        assert!(!Kind::Setup.matches("Misty-0.3.1.zip"));
        assert!(Kind::Portable.matches("Misty-0.3.1.zip"));
        assert!(!Kind::Portable.matches("Misty-Setup-0.3.1.zip"));
    }

    /// GitHub injoignable : la première page attend au plus le délai d'une requête, les suivantes
    /// répondent tout de suite (pas de nouvel essai à chaque visite).
    #[tokio::test]
    async fn panne_ne_bloque_pas_les_pages() {
        let dossier = std::env::temp_dir().join("misty-site-test-panne");
        let _ = std::fs::create_dir_all(&dossier);
        let _ = std::fs::remove_file(dossier.join("misty-site-releases.json"));
        // SAFETY : variable lue seulement par ce test, avant toute autre tâche.
        unsafe { std::env::set_var("MISTY_CACHE_DIR", &dossier) };
        let gh = GitHub::new("http://127.0.0.1:9".into(), "x/y".into(), None);
        let debut = Instant::now();
        assert!(gh.releases().await.is_none());
        assert!(debut.elapsed() < DELAI_REQUETE + Duration::from_secs(2));
        let debut = Instant::now();
        for _ in 0..50 {
            assert!(gh.releases().await.is_none());
        }
        assert!(debut.elapsed() < Duration::from_millis(50));
    }
}
