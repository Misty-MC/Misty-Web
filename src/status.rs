//! Page de statut « faite maison » : le site vérifie lui-même, toutes les minutes, les services dont
//! Misty a besoin (Mojang, connexion Microsoft, Modrinth, CurseForge, mod loaders, GitHub…).
//!
//! Chaque vérification est une simple requête (sans suivre les redirections) avec un délai maximal :
//! - en ligne : réponse sous 2,5 s, statut < 500 (un 403 ou un 404 prouve que le serveur répond) ;
//! - lent : réponse plus lente, ou 429 (trop de requêtes) ;
//! - hors ligne : pas de réponse, délai dépassé, ou erreur 5xx.
//!
//! On garde les 90 dernières vérifications de chaque service (1 h 30) pour la barre d'historique.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

const INTERVALLE: Duration = Duration::from_secs(60);
const DELAI: Duration = Duration::from_secs(6);
const LENT: Duration = Duration::from_millis(2500);
pub const HISTORIQUE: usize = 90;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

/// Ce que la page affiche pour un service.
#[derive(Clone, Debug)]
pub struct Ligne {
    pub index: usize,
    pub actuel: Option<Mesure>,
    pub historique: Vec<Option<Etat>>,
    /// Disponibilité sur l'historique (en ligne ou lent), en %
    pub dispo: Option<f32>,
}

pub struct Status {
    http: reqwest::Client,
    mesures: Mutex<Vec<VecDeque<Mesure>>>,
    derniere: Mutex<Option<SystemTime>>,
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

impl Status {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent("misty-site-status (https://misty-launcher.online/status)")
            .connect_timeout(Duration::from_secs(4))
            .timeout(DELAI)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client HTTP");
        Self {
            http,
            mesures: Mutex::new(vec![VecDeque::new(); SERVICES.len()]),
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
        let mut mesures = self.mesures.lock().unwrap();
        for (i, m) in resultats.into_iter().enumerate() {
            let file = &mut mesures[i];
            if file.len() == HISTORIQUE {
                file.pop_front();
            }
            file.push_back(m);
        }
        *self.derniere.lock().unwrap() = Some(SystemTime::now());
    }

    pub fn lignes(&self) -> Vec<Ligne> {
        let mesures = self.mesures.lock().unwrap();
        mesures
            .iter()
            .enumerate()
            .map(|(index, file)| {
                let mut historique: Vec<Option<Etat>> = vec![None; HISTORIQUE - file.len()];
                historique.extend(file.iter().map(|m| Some(m.etat)));
                let dispo = (!file.is_empty()).then(|| {
                    file.iter().filter(|m| m.etat != Etat::HorsLigne).count() as f32 * 100.0 / file.len() as f32
                });
                Ligne { index, actuel: file.back().copied(), historique, dispo }
            })
            .collect()
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

    #[test]
    fn historique_complete_a_gauche() {
        let s = Status::new();
        s.mesures.lock().unwrap()[0].push_back(Mesure { etat: Etat::EnLigne, latence: None });
        let l = &s.lignes()[0];
        assert_eq!(l.historique.len(), HISTORIQUE);
        assert_eq!(l.historique.last(), Some(&Some(Etat::EnLigne)));
        assert_eq!(l.dispo, Some(100.0));
        assert!(s.lignes()[1].dispo.is_none());
    }
}
