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
//! - une ligne de journal quand GitHub tombe, une quand il revient (au lieu d'une par visite) ;
//! - token refusé (401/403) : on réessaie sans token ;
//! - API indisponible (quota dépassé, panne) : on lit le flux Atom public des releases
//!   (`github.com/<repo>/releases.atom`, qui n'est pas soumis au quota de l'API), complété par la
//!   liste des fichiers de la page de la release ;
//! - les fichiers de la dernière version sont copiés sur le serveur (`miroir`) : les
//!   téléchargements ne dépendent plus de GitHub.

use serde::{Deserialize, Serialize};
use crate::miroir::Miroir;
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
    /// `https://github.com` (flux Atom et pages des releases)
    web_base: String,
    repo: String,
    token: Option<String>,
    etat: Mutex<Etat>,
    /// Une seule requête vers GitHub à la fois
    requete: tokio::sync::Mutex<()>,
    fichier_cache: PathBuf,
    pub miroir: Miroir,
}

enum Reponse {
    Nouvelle(Vec<Release>, Option<String>),
    Inchangee,
}

impl GitHub {
    pub fn new(api_base: String, web_base: String, repo: String, token: Option<String>) -> Self {
        let http = crate::http()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(5))
            .timeout(DELAI_REQUETE)
            .build()
            .expect("client HTTP");
        let dossier = std::env::var_os("MISTY_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let fichier_cache = dossier.join("misty-site-releases.json");
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
            web_base,
            repo,
            token,
            etat: Mutex::new(etat),
            requete: tokio::sync::Mutex::new(()),
            fichier_cache,
            miroir: Miroir::new(dossier.join("misty-site-downloads")),
        }
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Dernière liste connue, sans jamais contacter GitHub.
    pub fn en_cache(&self) -> Option<Arc<Vec<Release>>> {
        self.etat.lock().unwrap().liste.clone()
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
            // copie locale des fichiers de la dernière version (ne télécharge que ce qui manque)
            if let Some(r) = self.en_cache().as_deref().and_then(|l| latest(l)).cloned() {
                self.miroir.synchroniser(&r).await;
            }
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

    /// API (avec token, puis sans si GitHub le refuse), sinon flux Atom public.
    async fn fetch(&self, etag: Option<&str>) -> Result<Reponse, String> {
        let api = match self.fetch_api(etag, true).await {
            Err(e) if self.token.is_some() && matches!(e.status().map(|s| s.as_u16()), Some(401 | 403)) => {
                self.fetch_api(etag, false).await
            }
            r => r,
        };
        match api {
            Ok(r) => Ok(r),
            Err(e_api) => match self.fetch_atom().await {
                Ok(liste) if !liste.is_empty() => Ok(Reponse::Nouvelle(liste, None)),
                Ok(_) => Err(format!("{e_api} ; flux Atom vide")),
                Err(e_atom) => Err(format!("{e_api} ; flux Atom : {e_atom}")),
            },
        }
    }

    async fn fetch_api(&self, etag: Option<&str>, avec_token: bool) -> Result<Reponse, reqwest::Error> {
        let url = format!(
            "{}/repos/{}/releases?per_page=100",
            self.api_base, self.repo
        );
        let mut req = self
            .http
            .get(url)
            .header("Accept", "application/vnd.github+json");
        if let (Some(t), true) = (&self.token, avec_token) {
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

    /// Secours sans API : versions lues dans le flux Atom. Une version déjà connue garde ses
    /// données complètes ; pour une nouvelle, on lit la liste de ses fichiers sur sa page.
    async fn fetch_atom(&self) -> Result<Vec<Release>, reqwest::Error> {
        let url = format!("{}/{}/releases.atom", self.web_base, self.repo);
        let xml = self.http.get(url).send().await?.error_for_status()?.text().await?;
        let connues = self.en_cache();
        let mut liste = Vec::new();
        for (i, mut r) in parse_atom(&xml).into_iter().enumerate() {
            if let Some(c) = connues.as_deref().and_then(|l| l.iter().find(|c| c.tag_name == r.tag_name)) {
                liste.push(c.clone());
                continue;
            }
            // fichiers des 3 versions les plus récentes seulement (une requête chacune)
            if i < 3 {
                r.assets = self.assets_web(&r.tag_name).await.unwrap_or_default();
            }
            liste.push(r);
        }
        Ok(liste)
    }

    /// Fichiers d'une release, lus sur sa page (`expanded_assets`), tailles par requête HEAD.
    async fn assets_web(&self, tag: &str) -> Result<Vec<Asset>, reqwest::Error> {
        let url = format!("{}/{}/releases/expanded_assets/{tag}", self.web_base, self.repo);
        let html = self.http.get(url).send().await?.error_for_status()?.text().await?;
        let mut assets = Vec::new();
        for nom in noms_assets(&html, &self.repo, tag) {
            if Kind::Setup.matches(&nom) || Kind::Portable.matches(&nom) {
                let url = format!("{}/{}/releases/download/{tag}/{nom}", self.web_base, self.repo);
                let size = match self.http.head(&url).send().await {
                    // en-tête lu directement : pour une requête HEAD, le corps est vide
                    Ok(r) if r.status().is_success() => r
                        .headers()
                        .get(reqwest::header::CONTENT_LENGTH)
                        .and_then(|v| v.to_str().ok()?.parse().ok())
                        .unwrap_or(0),
                    _ => 0,
                };
                assets.push(Asset { name: nom, size, browser_download_url: url, content_type: None });
            }
        }
        Ok(assets)
    }
}

/// Texte entre la première occurrence de `debut` et le `fin` suivant.
fn entre<'a>(s: &'a str, debut: &str, fin: &str) -> Option<&'a str> {
    let i = s.find(debut)? + debut.len();
    let j = s[i..].find(fin)? + i;
    Some(&s[i..j])
}

fn deseschapper(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&amp;", "&")
}

/// HTML des notes (flux Atom) → Markdown simple : titres, listes, gras, code, liens.
fn html_vers_markdown(html: &str) -> String {
    let mut s = html.replace('\n', " ");
    for (a, b) in [
        ("<h1>", "\n\n# "), ("<h2>", "\n\n## "), ("<h3>", "\n\n### "), ("<h4>", "\n\n#### "),
        ("</h1>", "\n"), ("</h2>", "\n"), ("</h3>", "\n"), ("</h4>", "\n"),
        ("<li>", "\n- "), ("</ul>", "\n"), ("</ol>", "\n"), ("<p>", "\n\n"), ("</p>", "\n"),
        ("<br>", "\n"), ("<strong>", "**"), ("</strong>", "**"), ("<em>", "*"), ("</em>", "*"),
        ("<code>", "`"), ("</code>", "`"), ("<hr>", "\n\n---\n"),
    ] {
        s = s.replace(a, b);
    }
    // liens : <a href="url">texte</a> → [texte](url)
    let mut out = String::new();
    let mut reste = s.as_str();
    while let Some(i) = reste.find("<a ") {
        out.push_str(&reste[..i]);
        let bloc = &reste[i..];
        let href = entre(bloc, "href=\"", "\"").unwrap_or("");
        let (texte, suite) = match (bloc.find('>'), bloc.find("</a>")) {
            (Some(o), Some(f)) if o < f => (&bloc[o + 1..f], &bloc[f + 4..]),
            _ => ("", &bloc[3..]),
        };
        out.push_str(&format!("[{texte}]({href})"));
        reste = suite;
    }
    out.push_str(reste);
    // autres balises : retirées
    let mut texte = String::new();
    let mut dans_balise = false;
    for c in out.chars() {
        match c {
            '<' => dans_balise = true,
            '>' if dans_balise => dans_balise = false,
            _ if !dans_balise => texte.push(c),
            _ => {}
        }
    }
    let texte = deseschapper(&texte);
    let lignes: Vec<&str> = texte.lines().map(str::trim).collect();
    let mut md = lignes.join("\n");
    while md.contains("\n\n\n") {
        md = md.replace("\n\n\n", "\n\n");
    }
    md.trim().to_string()
}

/// Versions du flux Atom de GitHub, les plus récentes d'abord.
fn parse_atom(xml: &str) -> Vec<Release> {
    xml.split("<entry>")
        .skip(1)
        .filter_map(|e| {
            let lien = entre(e, "<link rel=\"alternate\" type=\"text/html\" href=\"", "\"")?;
            let tag = lien.rsplit("/releases/tag/").next().filter(|t| *t != lien)?.to_string();
            let titre = entre(e, "<title>", "</title>").map(deseschapper);
            let notes = entre(e, "<content type=\"html\">", "</content>").map(|h| html_vers_markdown(&deseschapper(h)));
            Some(Release {
                html_url: lien.to_string(),
                name: titre.filter(|t| !t.trim().is_empty()),
                body: notes,
                published_at: entre(e, "<updated>", "</updated>").map(str::to_string),
                prerelease: false,
                draft: false,
                assets: Vec::new(),
                tag_name: tag,
            })
        })
        .collect()
}

/// Noms des fichiers d'une release dans le HTML de sa page.
fn noms_assets(html: &str, repo: &str, tag: &str) -> Vec<String> {
    let motif = format!("/{repo}/releases/download/{tag}/");
    let mut noms: Vec<String> = Vec::new();
    for morceau in html.split(&motif).skip(1) {
        let nom: String = morceau.chars().take_while(|c| !matches!(c, '"' | '\'' | '<' | ' ' | '?')).collect();
        if !nom.is_empty() && !noms.contains(&nom) {
            noms.push(nom);
        }
    }
    noms
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
    fn flux_atom() {
        let xml = r#"<?xml version="1.0"?><feed><title>Release notes from Misty</title>
  <entry>
    <id>tag:github.com,2008:Repository/1/v0.3.4</id>
    <updated>2026-10-20T12:00:00Z</updated>
    <link rel="alternate" type="text/html" href="https://github.com/Misty-MC/Misty/releases/tag/v0.3.4"/>
    <title>Misty 0.3.4</title>
    <content type="html">&lt;h2&gt;Nouveautés&lt;/h2&gt;
&lt;ul&gt;
&lt;li&gt;Thème &lt;strong&gt;nuit&lt;/strong&gt; &amp;amp; paysage&lt;/li&gt;
&lt;li&gt;Voir &lt;a href=&quot;https://misty-launcher.online&quot;&gt;le site&lt;/a&gt;&lt;/li&gt;
&lt;/ul&gt;</content>
  </entry>
  <entry>
    <updated>2026-10-01T12:00:00Z</updated>
    <link rel="alternate" type="text/html" href="https://github.com/Misty-MC/Misty/releases/tag/v0.3.3"/>
    <title>v0.3.3</title>
    <content type="html"></content>
  </entry></feed>"#;
        let l = parse_atom(xml);
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].tag_name, "v0.3.4");
        assert_eq!(l[0].title(), "Misty 0.3.4");
        assert_eq!(l[0].published_at.as_deref(), Some("2026-10-20T12:00:00Z"));
        assert_eq!(l[0].notes(), "## Nouveautés\n\n- Thème **nuit** & paysage\n- Voir [le site](https://misty-launcher.online)");
        assert_eq!(l[1].tag_name, "v0.3.3");
    }

    #[test]
    fn fichiers_de_la_page() {
        let html = r#"<a href="/Misty-MC/Misty/releases/download/v0.3.4/Misty-Setup-0.3.4.exe" rel="nofollow">
            <a href="/Misty-MC/Misty/releases/download/v0.3.4/Misty-0.3.4.zip">
            <a href="/Misty-MC/Misty/releases/download/v0.3.4/Misty-0.3.4.zip">
            <a href="/Misty-MC/Misty/archive/refs/tags/v0.3.4.zip">"#;
        assert_eq!(noms_assets(html, "Misty-MC/Misty", "v0.3.4"), ["Misty-Setup-0.3.4.exe", "Misty-0.3.4.zip"]);
    }

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
        let gh = GitHub::new("http://127.0.0.1:9".into(), "http://127.0.0.1:9".into(), "x/y".into(), None);
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
