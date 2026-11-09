//! Page de statut « faite maison » : le site vérifie lui-même, toutes les minutes, les services dont
//! Misty a besoin (Mojang, connexion Microsoft, Modrinth, CurseForge, mod loaders, GitHub, Horizon…).
//!
//! Chaque vérification est une simple requête (sans suivre les redirections) avec un délai maximal :
//! - en ligne : réponse sous 2,5 s, statut < 500 (un 403 ou un 404 prouve que le serveur répond) ;
//! - lent : réponse plus lente, ou 429 (trop de requêtes) ;
//! - hors ligne : pas de réponse, délai dépassé, ou erreur 5xx.
//!
//! Les vérifications sont gardées une par une pour l'historique affiché (une case = une minute, sur
//! la dernière heure) et comptées par jour (UTC) pour la disponibilité sur 30 jours.
//! Un incident s'ouvre après plusieurs vérifications ratées de suite et se ferme quand le service
//! répond de nouveau ; on garde ceux des 90 derniers jours. Tout est enregistré sur le disque
//! (`MISTY_CACHE_DIR`, sinon le dossier temporaire) pour survivre aux redémarrages.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};
use time::{Date, OffsetDateTime};

const INTERVALLE: Duration = Duration::from_secs(60);
const DELAI: Duration = Duration::from_secs(6);
const LENT: Duration = Duration::from_millis(2500);
/// Jours comptés pour la disponibilité
pub const JOURS: usize = 30;
/// Cases de l'historique affiché : une par minute
pub const MINUTES: usize = 60;
/// Durée de l'historique des incidents (jours)
pub const JOURS_INCIDENTS: i64 = 90;
/// Vérifications ratées de suite avant d'ouvrir un incident (hors ligne, puis lent)
const SEUIL_PANNE: u32 = 2;
const SEUIL_LENTEUR: u32 = 5;
/// Vérifications réussies de suite pour le refermer
const SEUIL_RETOUR: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Etat {
    EnLigne,
    Lent,
    HorsLigne,
}

impl Etat {
    pub fn slug(self) -> &'static str {
        match self {
            Etat::EnLigne => "up",
            Etat::Lent => "slow",
            Etat::HorsLigne => "down",
        }
    }
}

/// Groupe d'affichage.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Groupe {
    Misty,
    Minecraft,
    Mods,
}

pub struct Service {
    pub nom: &'static str,
    pub groupe: Groupe,
    pub url: &'static str,
    /// Utilisé pour… (clé de texte, affichée sous le nom)
    pub role: Role,
}

/// À quoi sert le service dans Misty (texte traduit dans la page).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Telechargement,
    Notes,
    Horizon,
    Comptes,
    Connexion,
    Skins,
    Versions,
    Ressources,
    Bibliotheques,
    Mods,
    Loaders,
}

pub const SERVICES: &[Service] = &[
    Service { nom: "GitHub Releases", groupe: Groupe::Misty, url: "https://github.com/Misty-MC/Misty/releases/latest", role: Role::Telechargement },
    Service { nom: "GitHub API", groupe: Groupe::Misty, url: "https://www.githubstatus.com/api/v2/status.json", role: Role::Notes },
    Service { nom: "Horizon", groupe: Groupe::Misty, url: "https://github.com/horizon-rust/horizon/releases", role: Role::Horizon },
    Service { nom: "Mojang API", groupe: Groupe::Minecraft, url: "https://api.mojang.com/users/profiles/minecraft/Notch", role: Role::Comptes },
    Service { nom: "Session Server", groupe: Groupe::Minecraft, url: "https://sessionserver.mojang.com/session/minecraft/profile/069a79f444e94726a5befca90e38aaf5", role: Role::Skins },
    Service { nom: "Microsoft Login", groupe: Groupe::Minecraft, url: "https://login.live.com/", role: Role::Connexion },
    Service { nom: "Minecraft Services", groupe: Groupe::Minecraft, url: "https://api.minecraftservices.com/", role: Role::Connexion },
    Service { nom: "Piston Meta", groupe: Groupe::Minecraft, url: "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json", role: Role::Versions },
    Service { nom: "Minecraft Assets", groupe: Groupe::Minecraft, url: "https://resources.download.minecraft.net/", role: Role::Ressources },
    Service { nom: "Minecraft Libraries", groupe: Groupe::Minecraft, url: "https://libraries.minecraft.net/", role: Role::Bibliotheques },
    Service { nom: "Modrinth API", groupe: Groupe::Mods, url: "https://api.modrinth.com/", role: Role::Mods },
    Service { nom: "CurseForge API", groupe: Groupe::Mods, url: "https://api.curseforge.com/", role: Role::Mods },
    Service { nom: "Fabric Meta", groupe: Groupe::Mods, url: "https://meta.fabricmc.net/v2/versions/loader", role: Role::Loaders },
    Service { nom: "Forge Maven", groupe: Groupe::Mods, url: "https://maven.minecraftforge.net/", role: Role::Loaders },
    Service { nom: "NeoForge Maven", groupe: Groupe::Mods, url: "https://maven.neoforged.net/", role: Role::Loaders },
    Service { nom: "Quilt Meta", groupe: Groupe::Mods, url: "https://meta.quiltmc.org/v3/versions/loader", role: Role::Loaders },
];

/// Une vérification.
#[derive(Clone, Copy, Debug)]
pub struct Mesure {
    pub etat: Etat,
    pub latence: Option<Duration>,
}

/// Une vérification gardée pour l'historique à la minute.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Minute {
    /// Secondes Unix
    pub t: i64,
    pub etat: Etat,
    /// Latence en millisecondes
    pub ms: Option<u32>,
}

/// Vérifications d'un service pendant un jour (UTC).
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Jour {
    pub up: u32,
    pub slow: u32,
    pub down: u32,
}

impl Jour {
    pub fn total(&self) -> u32 {
        self.up + self.slow + self.down
    }
    /// Disponibilité (en ligne ou lent), en %
    pub fn dispo(&self) -> f32 {
        (self.up + self.slow) as f32 * 100.0 / self.total().max(1) as f32
    }
    /// Couleur de la case : en ligne, perturbé, ou en panne.
    pub fn etat(&self) -> Etat {
        let lent = self.slow as f32 * 100.0 / self.total().max(1) as f32;
        let dispo = self.dispo();
        if dispo >= 99.5 && lent < 10.0 {
            Etat::EnLigne
        } else if dispo >= 95.0 {
            Etat::Lent
        } else {
            Etat::HorsLigne
        }
    }
    fn plus(self, j: Jour) -> Jour {
        Jour { up: self.up + j.up, slow: self.slow + j.slow, down: self.down + j.down }
    }
}

/// Un incident : un service hors ligne (panne) ou lent (lenteur) plusieurs vérifications de suite.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Incident {
    pub service: String,
    pub panne: bool,
    /// Secondes Unix
    pub debut: i64,
    pub fin: Option<i64>,
}

#[derive(Default, Serialize, Deserialize)]
struct Donnees {
    /// service → date (AAAA-MM-JJ) → compteurs
    jours: HashMap<String, HashMap<String, Jour>>,
    /// service → dernières vérifications (une par minute)
    #[serde(default)]
    minutes: HashMap<String, VecDeque<Minute>>,
    incidents: Vec<Incident>,
}

/// Ce que la page affiche pour un service.
#[derive(Clone, Debug)]
pub struct Ligne {
    pub index: usize,
    pub actuel: Option<Mesure>,
    /// Les 30 derniers jours, du plus ancien à aujourd'hui
    pub jours: Vec<(Date, Option<Jour>)>,
    /// Les 60 dernières minutes, de la plus ancienne à maintenant (début de la minute, mesure)
    pub minutes: Vec<(i64, Option<Minute>)>,
    /// Disponibilité sur 30 jours, en %
    pub dispo: Option<f32>,
}

/// Suite de vérifications du même genre, pour ouvrir ou fermer un incident.
#[derive(Default, Clone, Copy)]
struct Serie {
    panne: u32,
    lenteur: u32,
    retour: u32,
}

pub struct Status {
    http: reqwest::Client,
    fichier: PathBuf,
    actuel: Mutex<Vec<Option<Mesure>>>,
    series: Mutex<Vec<Serie>>,
    donnees: Mutex<Donnees>,
    derniere: Mutex<Option<SystemTime>>,
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

fn aujourd_hui() -> Date {
    OffsetDateTime::now_utc().date()
}

impl Status {
    pub fn new() -> Self {
        let dossier = std::env::var_os("MISTY_CACHE_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        Self::avec_fichier(dossier.join("misty-site-status.json"))
    }

    fn avec_fichier(fichier: PathBuf) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("misty-site-status (https://misty-launcher.online/status)")
            .connect_timeout(Duration::from_secs(4))
            .timeout(DELAI)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client HTTP");
        let donnees = std::fs::read(&fichier).ok().and_then(|o| serde_json::from_slice(&o).ok()).unwrap_or_default();
        Self {
            http,
            fichier,
            actuel: Mutex::new(vec![None; SERVICES.len()]),
            series: Mutex::new(vec![Serie::default(); SERVICES.len()]),
            donnees: Mutex::new(donnees),
            derniere: Mutex::new(None),
        }
    }

    pub async fn verifier_en_boucle(self: std::sync::Arc<Self>) {
        loop {
            self.verifier_tout().await;
            tokio::time::sleep(INTERVALLE).await;
        }
    }

    async fn verifier_tout(&self) {
        // toutes les vérifications en même temps
        let poignees: Vec<_> = SERVICES
            .iter()
            .map(|s| {
                let (http, url) = (self.http.clone(), s.url);
                tokio::spawn(async move { verifier(&http, url).await })
            })
            .collect();
        let mut resultats = Vec::with_capacity(poignees.len());
        for p in poignees {
            resultats.push(p.await.unwrap_or(Mesure { etat: Etat::HorsLigne, latence: None }));
        }
        self.enregistrer(&resultats, OffsetDateTime::now_utc());
        // écriture sur le disque hors du verrou
        let json = serde_json::to_vec(&*self.donnees.lock().unwrap()).ok();
        if let Some(json) = json {
            let fichier = self.fichier.clone();
            let _ = tokio::task::spawn_blocking(move || std::fs::write(fichier, json)).await;
        }
    }

    /// Compte un tour de vérifications (un résultat par service, dans l'ordre de SERVICES).
    fn enregistrer(&self, resultats: &[Mesure], maintenant: OffsetDateTime) {
        let jour = maintenant.date().to_string();
        let horodatage = maintenant.unix_timestamp();
        let mut donnees = self.donnees.lock().unwrap();
        let mut series = self.series.lock().unwrap();
        for (i, m) in resultats.iter().enumerate() {
            let nom = SERVICES[i].nom;
            let compteurs = donnees.jours.entry(nom.to_string()).or_default().entry(jour.clone()).or_default();
            match m.etat {
                Etat::EnLigne => compteurs.up += 1,
                Etat::Lent => compteurs.slow += 1,
                Etat::HorsLigne => compteurs.down += 1,
            }
            let minutes = donnees.minutes.entry(nom.to_string()).or_default();
            let ms = m.latence.map(|d| d.as_millis().min(u32::MAX as u128) as u32);
            minutes.push_back(Minute { t: horodatage, etat: m.etat, ms });
            while minutes.len() > MINUTES * 2 {
                minutes.pop_front();
            }
            // incidents
            let s = &mut series[i];
            match m.etat {
                Etat::EnLigne => {
                    s.retour += 1;
                    s.panne = 0;
                    s.lenteur = 0;
                }
                Etat::Lent => {
                    s.lenteur += 1;
                    s.panne = 0;
                    s.retour = 0;
                }
                Etat::HorsLigne => {
                    s.panne += 1;
                    s.retour = 0;
                }
            }
            let ouvert = donnees.incidents.iter_mut().rev().find(|x| x.service == nom && x.fin.is_none());
            match ouvert {
                Some(inc) => {
                    if s.retour >= SEUIL_RETOUR {
                        inc.fin = Some(horodatage);
                    } else if s.panne >= SEUIL_PANNE {
                        inc.panne = true; // une lenteur devenue panne
                    }
                }
                None => {
                    let panne = s.panne >= SEUIL_PANNE;
                    if panne || s.lenteur >= SEUIL_LENTEUR {
                        let suite = if panne { s.panne } else { s.lenteur } as i64;
                        let debut = horodatage - INTERVALLE.as_secs() as i64 * (suite - 1);
                        donnees.incidents.push(Incident { service: nom.to_string(), panne, debut, fin: None });
                    }
                }
            }
        }
        // ménage : jours au-delà de l'historique, incidents de plus de 90 jours
        let limite = (maintenant.date() - time::Duration::days(JOURS as i64)).to_string();
        for jours in donnees.jours.values_mut() {
            jours.retain(|d, _| *d > limite);
        }
        let limite_incidents = horodatage - JOURS_INCIDENTS * 86_400;
        donnees.incidents.retain(|x| x.fin.unwrap_or(i64::MAX) >= limite_incidents);
        drop(donnees);
        let mut actuel = self.actuel.lock().unwrap();
        for (i, m) in resultats.iter().enumerate() {
            actuel[i] = Some(*m);
        }
        *self.derniere.lock().unwrap() = Some(SystemTime::now());
    }

    pub fn lignes(&self) -> Vec<Ligne> {
        self.lignes_a(OffsetDateTime::now_utc().unix_timestamp())
    }

    fn lignes_a(&self, maintenant: i64) -> Vec<Ligne> {
        let donnees = self.donnees.lock().unwrap();
        let actuel = self.actuel.lock().unwrap();
        let today = OffsetDateTime::from_unix_timestamp(maintenant).map(|t| t.date()).unwrap_or_else(|_| aujourd_hui());
        let minute = maintenant.div_euclid(60);
        SERVICES
            .iter()
            .enumerate()
            .map(|(index, s)| {
                let compteurs = donnees.jours.get(s.nom);
                let jours: Vec<(Date, Option<Jour>)> = (0..JOURS)
                    .rev()
                    .map(|n| {
                        let d = today - time::Duration::days(n as i64);
                        (d, compteurs.and_then(|j| j.get(&d.to_string())).copied())
                    })
                    .collect();
                let mesures = donnees.minutes.get(s.nom);
                let minutes: Vec<(i64, Option<Minute>)> = (0..MINUTES as i64)
                    .rev()
                    .map(|n| {
                        let m = minute - n;
                        (m * 60, mesures.and_then(|v| v.iter().rev().find(|x| x.t.div_euclid(60) == m)).copied())
                    })
                    .collect();
                let somme = jours.iter().filter_map(|(_, j)| *j).fold(Jour::default(), Jour::plus);
                let dispo = (somme.total() > 0).then(|| somme.dispo());
                Ligne { index, actuel: actuel[index], jours, minutes, dispo }
            })
            .collect()
    }

    /// Disponibilité de tous les services sur 30 jours, en %.
    pub fn dispo_globale(&self) -> Option<f32> {
        let donnees = self.donnees.lock().unwrap();
        let somme = donnees.jours.values().flat_map(|j| j.values()).copied().fold(Jour::default(), Jour::plus);
        (somme.total() > 0).then(|| somme.dispo())
    }

    /// Incidents des 90 derniers jours, les plus récents d'abord.
    pub fn incidents(&self) -> Vec<Incident> {
        let mut liste = self.donnees.lock().unwrap().incidents.clone();
        liste.sort_by(|a, b| b.debut.cmp(&a.debut));
        liste
    }

    /// Secondes depuis la dernière vérification.
    pub fn age(&self) -> Option<u64> {
        self.derniere.lock().unwrap().and_then(|t| t.elapsed().ok()).map(|d| d.as_secs())
    }
}

/// Une vérification : une requête, sans suivre les redirections, avec un délai maximal.
async fn verifier(http: &reqwest::Client, url: &str) -> Mesure {
    let debut = Instant::now();
    match http.get(url).send().await {
        Ok(r) => {
            let latence = debut.elapsed();
            let code = r.status().as_u16();
            let etat = if code >= 500 {
                Etat::HorsLigne
            } else if code == 429 || latence > LENT {
                Etat::Lent
            } else {
                Etat::EnLigne
            };
            Mesure { etat, latence: Some(latence) }
        }
        Err(_) => Mesure { etat: Etat::HorsLigne, latence: None },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tour(etat_premier: Etat) -> Vec<Mesure> {
        let mut v = vec![Mesure { etat: Etat::EnLigne, latence: None }; SERVICES.len()];
        v[0].etat = etat_premier;
        v
    }

    fn status_vide(nom: &str) -> Status {
        let f = std::env::temp_dir().join(nom);
        let _ = std::fs::remove_file(&f);
        Status::avec_fichier(f)
    }

    #[test]
    fn historique_par_minute() {
        let s = status_vide("misty-status-test-minutes.json");
        let t0 = OffsetDateTime::now_utc();
        s.enregistrer(&tour(Etat::HorsLigne), t0 - time::Duration::minutes(2));
        s.enregistrer(&tour(Etat::EnLigne), t0);
        let l = &s.lignes_a(t0.unix_timestamp())[0];
        assert_eq!(l.minutes.len(), MINUTES);
        let (debut, derniere) = *l.minutes.last().unwrap();
        assert_eq!(debut, t0.unix_timestamp().div_euclid(60) * 60);
        assert_eq!(derniere.map(|m| m.etat), Some(Etat::EnLigne));
        assert_eq!(l.minutes[MINUTES - 3].1.map(|m| m.etat), Some(Etat::HorsLigne));
        assert!(l.minutes[MINUTES - 2].1.is_none());
        assert!(l.minutes[0].1.is_none());
    }

    #[test]
    fn historique_par_jour() {
        let s = status_vide("misty-status-test-jours.json");
        s.enregistrer(&tour(Etat::EnLigne), OffsetDateTime::now_utc());
        let l = &s.lignes()[0];
        assert_eq!(l.jours.len(), JOURS);
        assert_eq!(l.jours.last().unwrap().0, aujourd_hui());
        assert_eq!(l.jours.last().unwrap().1, Some(Jour { up: 1, slow: 0, down: 0 }));
        assert!(l.jours[0].1.is_none());
        assert_eq!(l.dispo, Some(100.0));
        assert_eq!(s.dispo_globale(), Some(100.0));
    }

    #[test]
    fn incident_ouvert_puis_resolu() {
        let s = status_vide("misty-status-test-incidents.json");
        let t0 = OffsetDateTime::now_utc();
        let minute = time::Duration::minutes(1);
        s.enregistrer(&tour(Etat::HorsLigne), t0);
        assert!(s.incidents().is_empty(), "une seule erreur ne suffit pas");
        s.enregistrer(&tour(Etat::HorsLigne), t0 + minute);
        let inc = s.incidents();
        assert_eq!(inc.len(), 1);
        assert!(inc[0].panne && inc[0].fin.is_none());
        assert_eq!(inc[0].debut, t0.unix_timestamp());
        s.enregistrer(&tour(Etat::EnLigne), t0 + time::Duration::minutes(2));
        s.enregistrer(&tour(Etat::EnLigne), t0 + time::Duration::minutes(3));
        assert_eq!(s.incidents()[0].fin, Some((t0 + time::Duration::minutes(3)).unix_timestamp()));
    }

    #[test]
    fn couleur_des_jours() {
        assert_eq!(Jour { up: 1440, slow: 0, down: 0 }.etat(), Etat::EnLigne);
        assert_eq!(Jour { up: 1438, slow: 0, down: 2 }.etat(), Etat::EnLigne);
        assert_eq!(Jour { up: 1380, slow: 0, down: 60 }.etat(), Etat::Lent);
        assert_eq!(Jour { up: 700, slow: 0, down: 740 }.etat(), Etat::HorsLigne);
    }
}
