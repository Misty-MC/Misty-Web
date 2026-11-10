//! Réseau de Misty Client (le mod en jeu du launcher) : qui joue avec Misty en ce moment, et
//! avec quel cosmétique Misty, pour que les joueurs Misty se reconnaissent en jeu (nuage dans la
//! liste des joueurs et le chat, cape et élytres Misty).
//!
//! - `POST /api/client/login` : le jeu prouve son compte comme auprès d'un serveur Minecraft. Il
//!   « rejoint » un identifiant de serveur tiré au hasard auprès de Mojang (`joinServer`), le site
//!   le vérifie chez Mojang (`hasJoined`) et rend un jeton. Personne ne peut donc se faire passer
//!   pour un autre, et le jeton Minecraft du joueur ne quitte jamais son jeu.
//! - `POST /api/client/presence` (jeton) : signe de vie toutes les minutes, avec le cosmétique porté.
//! - `POST /api/client/players` (jeton) : parmi ces joueurs (ceux du serveur où l'on joue),
//!   lesquels sont en ligne avec Misty, et que portent-ils.
//!
//! Tout reste en mémoire : rien n'est écrit, les listes de joueurs demandées ne sont pas gardées,
//! aucune adresse IP n'est conservée (elle ne sert qu'à limiter le débit, une minute).

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rand::Rng;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::AppState;

/// Un joueur compte comme en ligne tant que son dernier signe date de moins de 3 minutes.
const EN_LIGNE: Duration = Duration::from_secs(180);
/// Un jeton sert tant qu'il donne des signes de vie ; sans signe pendant ce temps, il est oublié
/// (le jeu se reconnecte alors tout seul).
const JETON_INACTIF: Duration = Duration::from_secs(3600);
/// Requêtes acceptées par adresse et par minute (plusieurs joueurs derrière la même box).
const MAX_PAR_MINUTE: u32 = 240;
/// Connexions par adresse et par minute : chacune interroge Mojang.
const MAX_LOGIN_PAR_MINUTE: u32 = 12;
/// Joueurs demandés par requête.
const MAX_UUIDS: usize = 100;
/// Au-delà, les jetons inactifs sont oubliés, puis les nouvelles connexions refusées (protection mémoire).
const MAX_JETONS: usize = 200_000;
/// Cosmétiques Misty connus (les mêmes identifiants que le launcher et le mod).
const COSMETIQUES: &[&str] = &["misty_cape", "misty_elytra"];

struct Jeton {
    /// UUID du compte, 32 chiffres hexadécimaux en minuscules
    uuid: String,
    vu: Instant,
}

struct Joueur {
    vu: Instant,
    cosmetique: &'static str,
}

pub struct Client {
    /// Service de session de Mojang (`MOJANG_SESSION` pour les tests).
    mojang: String,
    /// jeton → compte
    jetons: Mutex<HashMap<String, Jeton>>,
    /// UUID → joueur en ligne
    joueurs: Mutex<HashMap<String, Joueur>>,
    /// (adresse, sorte de requête) → (début de la minute, compte), seulement pour limiter le débit
    debits: Mutex<HashMap<(String, bool), (Instant, u32)>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            mojang: std::env::var("MOJANG_SESSION").ok().filter(|v| !v.is_empty()).unwrap_or_else(|| "https://sessionserver.mojang.com".into()),
            jetons: Mutex::default(),
            joueurs: Mutex::default(),
            debits: Mutex::default(),
        }
    }

    fn autorise(&self, adresse: &str, login: bool) -> bool {
        let mut d = self.debits.lock().unwrap();
        if d.len() > 50_000 {
            d.retain(|_, (debut, _)| debut.elapsed() < Duration::from_secs(60));
        }
        let e = d.entry((adresse.to_string(), login)).or_insert((Instant::now(), 0));
        if e.0.elapsed() >= Duration::from_secs(60) {
            *e = (Instant::now(), 0);
        }
        e.1 += 1;
        e.1 <= if login { MAX_LOGIN_PAR_MINUTE } else { MAX_PAR_MINUTE }
    }

    /// Nouveau jeton pour ce compte (l'ancien, s'il y en avait un, ne sert plus).
    fn ouvrir(&self, uuid: &str) -> Option<String> {
        let mut jetons = self.jetons.lock().unwrap();
        jetons.retain(|_, j| j.uuid != uuid);
        if jetons.len() >= MAX_JETONS {
            jetons.retain(|_, j| j.vu.elapsed() < JETON_INACTIF);
            if jetons.len() >= MAX_JETONS {
                return None;
            }
        }
        let mut octets = [0u8; 32];
        rand::rng().fill(&mut octets);
        let jeton: String = octets.iter().map(|o| format!("{o:02x}")).collect();
        jetons.insert(jeton.clone(), Jeton { uuid: uuid.to_string(), vu: Instant::now() });
        Some(jeton)
    }

    /// Compte du jeton présenté (`Authorization: Bearer …`), s'il est encore connu.
    fn compte(&self, entetes: &HeaderMap) -> Option<String> {
        let jeton = entetes.get(header::AUTHORIZATION)?.to_str().ok()?.strip_prefix("Bearer ")?;
        let mut jetons = self.jetons.lock().unwrap();
        let j = jetons.get_mut(jeton).filter(|j| j.vu.elapsed() < JETON_INACTIF)?;
        j.vu = Instant::now();
        Some(j.uuid.clone())
    }

    fn signe(&self, uuid: String, cosmetique: &'static str) {
        let mut joueurs = self.joueurs.lock().unwrap();
        if joueurs.len() > MAX_JETONS {
            joueurs.retain(|_, j| j.vu.elapsed() < EN_LIGNE);
        }
        joueurs.insert(uuid, Joueur { vu: Instant::now(), cosmetique });
    }

    fn quitte(&self, uuid: &str) {
        self.joueurs.lock().unwrap().remove(uuid);
    }

    /// Parmi ces UUID, ceux qui sont en ligne avec Misty : `{ "<uuid avec tirets>": { "cosmetic": "…" } }`.
    fn en_ligne(&self, uuids: &[String]) -> Map<String, Value> {
        let joueurs = self.joueurs.lock().unwrap();
        uuids
            .iter()
            .filter_map(|u| {
                let j = joueurs.get(u).filter(|j| j.vu.elapsed() < EN_LIGNE)?;
                Some((avec_tirets(u), json!({ "cosmetic": j.cosmetique })))
            })
            .collect()
    }
}

/// UUID écrit avec ou sans tirets → 32 chiffres hexadécimaux en minuscules.
fn uuid_normalise(texte: &str) -> Option<String> {
    let hex: String = texte.chars().filter(|c| *c != '-').collect::<String>().to_ascii_lowercase();
    (hex.len() == 32 && hex.bytes().all(|b| b.is_ascii_hexdigit())).then_some(hex)
}

fn avec_tirets(hex: &str) -> String {
    format!("{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..])
}

fn cosmetique_connu(texte: Option<&str>) -> &'static str {
    COSMETIQUES.iter().copied().find(|c| Some(*c) == texte).unwrap_or("")
}

fn erreur(code: StatusCode, message: &str) -> Response {
    (code, Json(json!({ "error": message }))).into_response()
}

/// `POST /api/client/login` : `{ "name": "<pseudo>", "serverId": "<40 chiffres hexadécimaux>" }`
/// → `{ "token": "…", "uuid": "…" }`. 403 si Mojang ne confirme pas ce compte.
pub async fn login(State(state): State<Arc<AppState>>, entetes: HeaderMap, Json(corps): Json<Value>) -> Response {
    let c = &state.client;
    if !c.autorise(&crate::presence::adresse(&entetes), true) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    let nom = corps["name"].as_str().filter(|n| (1..=16).contains(&n.len()) && n.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'));
    let serveur = corps["serverId"].as_str().filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()));
    let (Some(nom), Some(serveur)) = (nom, serveur) else {
        return erreur(StatusCode::BAD_REQUEST, "bad request");
    };

    let url = format!("{}/session/minecraft/hasJoined?username={nom}&serverId={serveur}", c.mojang);
    let reponse = match state.github.http().get(url).timeout(Duration::from_secs(10)).send().await {
        Ok(r) => r,
        Err(_) => return erreur(StatusCode::BAD_GATEWAY, "mojang unreachable"),
    };
    // 204 : Mojang ne connaît pas cette connexion (compte hors ligne, ou mauvais pseudo)
    if reponse.status() != reqwest::StatusCode::OK {
        return erreur(StatusCode::FORBIDDEN, "not verified");
    }
    let profil: Value = reponse.json().await.unwrap_or(Value::Null);
    let Some(uuid) = profil["id"].as_str().and_then(uuid_normalise) else {
        return erreur(StatusCode::BAD_GATEWAY, "bad mojang answer");
    };
    match c.ouvrir(&uuid) {
        Some(jeton) => Json(json!({ "token": jeton, "uuid": avec_tirets(&uuid) })).into_response(),
        None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

/// `POST /api/client/presence` (jeton) : `{ "cosmetic": "misty_cape" }`, ou `{ "bye": true }`
/// quand le jeu se ferme. 401 si le jeton n'est plus connu : le jeu se reconnecte.
pub async fn presence(State(state): State<Arc<AppState>>, entetes: HeaderMap, Json(corps): Json<Value>) -> Response {
    let c = &state.client;
    if !c.autorise(&crate::presence::adresse(&entetes), false) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    let Some(uuid) = c.compte(&entetes) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    if corps["bye"].as_bool() == Some(true) {
        c.quitte(&uuid);
    } else {
        c.signe(uuid, cosmetique_connu(corps["cosmetic"].as_str()));
    }
    StatusCode::NO_CONTENT.into_response()
}

/// `POST /api/client/players` (jeton) : `{ "uuids": ["…", …] }` (100 au plus)
/// → `{ "players": { "<uuid>": { "cosmetic": "misty_cape" } } }` : seulement ceux en ligne avec Misty.
pub async fn joueurs(State(state): State<Arc<AppState>>, entetes: HeaderMap, Json(corps): Json<Value>) -> Response {
    let c = &state.client;
    if !c.autorise(&crate::presence::adresse(&entetes), false) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    if c.compte(&entetes).is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(liste) = corps["uuids"].as_array().filter(|l| l.len() <= MAX_UUIDS) else {
        return erreur(StatusCode::BAD_REQUEST, "bad request");
    };
    let uuids: Vec<String> = liste.iter().filter_map(|u| u.as_str().and_then(uuid_normalise)).collect();
    Json(json!({ "players": c.en_ligne(&uuids) })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALEX: &str = "ec561538f3fd461daff5086b22154bce";

    fn entetes(jeton: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::AUTHORIZATION, format!("Bearer {jeton}").parse().unwrap());
        h
    }

    #[test]
    fn uuid_avec_ou_sans_tirets() {
        assert_eq!(uuid_normalise("EC561538-F3FD-461D-AFF5-086B22154BCE").as_deref(), Some(ALEX));
        assert_eq!(uuid_normalise(ALEX).as_deref(), Some(ALEX));
        assert_eq!(uuid_normalise("ec561538"), None);
        assert_eq!(uuid_normalise("zz561538f3fd461daff5086b22154bce"), None);
        assert_eq!(avec_tirets(ALEX), "ec561538-f3fd-461d-aff5-086b22154bce");
    }

    #[test]
    fn cosmetiques_inconnus_ignores() {
        assert_eq!(cosmetique_connu(Some("misty_cape")), "misty_cape");
        assert_eq!(cosmetique_connu(Some("<script>")), "");
        assert_eq!(cosmetique_connu(None), "");
    }

    #[test]
    fn jeton_puis_presence() {
        let c = Client::new();
        let jeton = c.ouvrir(ALEX).unwrap();
        assert_eq!(jeton.len(), 64);
        assert_eq!(c.compte(&entetes(&jeton)).as_deref(), Some(ALEX));
        assert_eq!(c.compte(&entetes("inconnu")), None);
        assert_eq!(c.compte(&HeaderMap::new()), None);

        // pas encore de signe de vie : pas en ligne
        assert!(c.en_ligne(&[ALEX.to_string()]).is_empty());
        c.signe(ALEX.to_string(), "misty_cape");
        let en_ligne = c.en_ligne(&[ALEX.to_string(), "0".repeat(32)]);
        assert_eq!(en_ligne.len(), 1);
        assert_eq!(en_ligne["ec561538-f3fd-461d-aff5-086b22154bce"]["cosmetic"], "misty_cape");

        c.quitte(ALEX);
        assert!(c.en_ligne(&[ALEX.to_string()]).is_empty());
    }

    #[test]
    fn une_nouvelle_connexion_remplace_l_ancienne() {
        let c = Client::new();
        let ancien = c.ouvrir(ALEX).unwrap();
        let nouveau = c.ouvrir(ALEX).unwrap();
        assert_ne!(ancien, nouveau);
        assert_eq!(c.compte(&entetes(&ancien)), None);
        assert_eq!(c.compte(&entetes(&nouveau)).as_deref(), Some(ALEX));
    }

    #[test]
    fn debit_limite() {
        let c = Client::new();
        assert!((0..MAX_LOGIN_PAR_MINUTE).all(|_| c.autorise("1.2.3.4", true)));
        assert!(!c.autorise("1.2.3.4", true));
        // les autres requêtes et les autres adresses ont leur propre compte
        assert!(c.autorise("1.2.3.4", false));
        assert!(c.autorise("5.6.7.8", true));
    }
}
