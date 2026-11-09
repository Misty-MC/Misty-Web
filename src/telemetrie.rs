//! Télémétrie du launcher (seulement chez les joueurs qui l'ont acceptée) et rapports de logs.
//!
//! - `POST /api/telemetry` : un événement (démarrage, partie lancée, partie finie ou plantée,
//!   mise à jour), ajouté à `telemetry/events-<jour>.jsonl`.
//! - `POST /api/telemetry/report` : les dernières lignes de la console d'une partie (envoyées
//!   par le joueur, ou automatiquement après un plantage), dans `telemetry/reports/<code>.json`.
//!   Réponse : le code du rapport, que le joueur donne sur Discord.
//! - `GET /api/telemetry/summary` et `GET /api/telemetry/report/{code}` : lecture, avec
//!   `Authorization: Bearer <TELEMETRY_TOKEN>` (sans cette variable, ces routes n'existent pas).
//!
//! Aucune adresse IP n'est gardée (elle sert seulement à limiter le débit, en mémoire). Les
//! fichiers de plus de 90 jours sont supprimés.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use time::OffsetDateTime;
use time::macros::format_description;

use crate::AppState;

/// Jours de conservation.
const GARDE_JOURS: i64 = 90;
/// Événements par heure et par adresse.
const MAX_EVENEMENTS: u32 = 240;
/// Rapports par heure et par adresse.
const MAX_RAPPORTS: u32 = 12;
/// Taille maximale d'un journal dans un rapport.
const MAX_LOG: usize = 700 * 1024;

pub struct Telemetrie {
    dossier: PathBuf,
    jeton: Option<String>,
    /// (adresse, type) → (début de la fenêtre, compte)
    debits: Mutex<HashMap<(String, bool), (Instant, u32)>>,
    /// Écritures dans les fichiers, une à la fois
    ecriture: tokio::sync::Mutex<()>,
}

impl Telemetrie {
    pub fn new() -> Self {
        let racine = std::env::var_os("MISTY_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        Self {
            dossier: racine.join("telemetry"),
            jeton: std::env::var("TELEMETRY_TOKEN").ok().filter(|t| t.len() >= 16),
            debits: Mutex::new(HashMap::new()),
            ecriture: tokio::sync::Mutex::new(()),
        }
    }

    /// Dossier des fichiers de télémétrie.
    pub fn dossier(&self) -> PathBuf {
        self.dossier.clone()
    }

    /// Ménage au démarrage puis chaque jour.
    pub async fn menage_en_boucle(self: Arc<Self>) {
        loop {
            self.menage();
            tokio::time::sleep(Duration::from_secs(24 * 3600)).await;
        }
    }

    fn menage(&self) {
        let limite = std::time::SystemTime::now() - Duration::from_secs(GARDE_JOURS as u64 * 86_400);
        for d in [self.dossier.clone(), self.dossier.join("reports")] {
            let Ok(lecture) = std::fs::read_dir(&d) else { continue };
            for e in lecture.flatten() {
                let vieux = e.metadata().and_then(|m| m.modified()).is_ok_and(|m| m < limite);
                if vieux && e.path().is_file() {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
    }

    /// false si l'adresse a trop envoyé cette heure-ci.
    fn autorise(&self, adresse: &str, rapport: bool) -> bool {
        let mut d = self.debits.lock().unwrap();
        if d.len() > 50_000 {
            d.retain(|_, (debut, _)| debut.elapsed() < Duration::from_secs(3600));
        }
        let entree = d.entry((adresse.to_string(), rapport)).or_insert((Instant::now(), 0));
        if entree.0.elapsed() >= Duration::from_secs(3600) {
            *entree = (Instant::now(), 0);
        }
        entree.1 += 1;
        entree.1 <= if rapport { MAX_RAPPORTS } else { MAX_EVENEMENTS }
    }

    fn lecture_autorisee(&self, entetes: &HeaderMap) -> bool {
        let Some(jeton) = &self.jeton else { return false };
        entetes
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .is_some_and(|v| egal_constant(v.as_bytes(), jeton.as_bytes()))
    }
}

/// Comparaison en temps constant (le jeton ne se devine pas à la durée de la réponse).
fn egal_constant(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Adresse du client (derrière Cloudflare et le proxy), seulement pour limiter le débit.
fn adresse(entetes: &HeaderMap) -> String {
    for nom in ["cf-connecting-ip", "x-forwarded-for", "x-real-ip"] {
        if let Some(v) = entetes.get(nom).and_then(|v| v.to_str().ok()) {
            if let Some(premiere) = v.split(',').next().map(str::trim).filter(|s| !s.is_empty()) {
                return premiere.to_string();
            }
        }
    }
    "?".into()
}

fn texte(v: &Value, cle: &str, max: usize) -> Option<String> {
    v.get(cle)?
        .as_str()
        .map(|s| s.chars().take(max).collect::<String>())
        .filter(|s| !s.is_empty())
}

/// Champs communs, recopiés un par un (rien d'autre n'est gardé).
fn contexte(v: &Value) -> Option<serde_json::Map<String, Value>> {
    let id = texte(v, "id", 64)?;
    let mut m = serde_json::Map::new();
    m.insert("id".into(), json!(id));
    for (cle, max) in [("version", 32), ("os", 120), ("langue", 12)] {
        if let Some(t) = texte(v, cle, max) {
            m.insert(cle.into(), json!(t));
        }
    }
    if let Some(ram) = v.get("ram_mo").and_then(Value::as_u64) {
        m.insert("ram_mo".into(), json!(ram));
    }
    m.insert(
        "received".into(),
        json!(OffsetDateTime::now_utc().unix_timestamp()),
    );
    Some(m)
}

fn jour() -> String {
    OffsetDateTime::now_utc()
        .format(format_description!("[year]-[month]-[day]"))
        .unwrap_or_default()
}

/// Code de rapport court, lisible à voix haute (sans 0/O ni 1/I).
fn code_rapport() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
    );
    let mut n = h.finish();
    let mut code = String::from("MR-");
    for _ in 0..7 {
        code.push(ALPHABET[(n % 32) as usize] as char);
        n /= 32;
    }
    code
}

fn code_valide(code: &str) -> bool {
    code.len() == 10
        && code.starts_with("MR-")
        && code[3..].bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
}

fn refus(statut: StatusCode, raison: &str) -> Response {
    (statut, Json(json!({ "error": raison }))).into_response()
}

/// `POST /api/telemetry`
pub async fn evenement(
    State(state): State<Arc<AppState>>,
    entetes: HeaderMap,
    Json(corps): Json<Value>,
) -> Response {
    let t = &state.telemetrie;
    if !t.autorise(&adresse(&entetes), false) {
        return refus(StatusCode::TOO_MANY_REQUESTS, "too many events");
    }
    let Some(mut ligne) = contexte(&corps) else {
        return refus(StatusCode::BAD_REQUEST, "missing id");
    };
    let Some(nom) = texte(&corps, "event", 40)
        .filter(|n| n.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'))
    else {
        return refus(StatusCode::BAD_REQUEST, "bad event");
    };
    ligne.insert("event".into(), json!(nom));
    // données de l'événement : un objet plat et court
    if let Some(Value::Object(d)) = corps.get("data") {
        let propre: serde_json::Map<String, Value> = d
            .iter()
            .take(24)
            .filter(|(k, v)| k.len() <= 32 && !v.is_object() && !v.is_array())
            .map(|(k, v)| {
                let v = match v {
                    Value::String(s) => json!(s.chars().take(300).collect::<String>()),
                    autre => autre.clone(),
                };
                (k.clone(), v)
            })
            .collect();
        ligne.insert("data".into(), Value::Object(propre));
    }
    let _verrou = t.ecriture.lock().await;
    let _ = tokio::fs::create_dir_all(&t.dossier).await;
    let fichier = t.dossier.join(format!("events-{}.jsonl", jour()));
    let mut texte = Value::Object(ligne).to_string();
    texte.push('\n');
    use tokio::io::AsyncWriteExt;
    match tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&fichier)
        .await
    {
        Ok(mut f) => {
            let _ = f.write_all(texte.as_bytes()).await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => refus(StatusCode::SERVICE_UNAVAILABLE, "storage"),
    }
}

/// `POST /api/telemetry/report`
pub async fn rapport(
    State(state): State<Arc<AppState>>,
    entetes: HeaderMap,
    Json(corps): Json<Value>,
) -> Response {
    let t = &state.telemetrie;
    if !t.autorise(&adresse(&entetes), true) {
        return refus(StatusCode::TOO_MANY_REQUESTS, "too many reports");
    }
    let Some(mut r) = contexte(&corps) else {
        return refus(StatusCode::BAD_REQUEST, "missing id");
    };
    let Some(log) = corps.get("log").and_then(Value::as_str) else {
        return refus(StatusCode::BAD_REQUEST, "missing log");
    };
    // la fin du journal est la plus utile (le plantage)
    let mut debut = log.len().saturating_sub(MAX_LOG);
    while !log.is_char_boundary(debut) {
        debut += 1;
    }
    for (cle, max) in [("reason", 32), ("game", 200)] {
        if let Some(v) = texte(&corps, cle, max) {
            r.insert(cle.into(), json!(v));
        }
    }
    r.insert("log".into(), json!(&log[debut..]));
    let code = code_rapport();
    r.insert("code".into(), json!(code));
    // salon de l'équipe sur Discord (en arrière-plan : le joueur a sa réponse tout de suite)
    {
        let discord = state.discord.clone();
        let copie = r.clone();
        tokio::spawn(async move { discord.rapport(&copie).await });
    }
    let dossier = t.dossier.join("reports");
    if tokio::fs::create_dir_all(&dossier).await.is_err()
        || tokio::fs::write(
            dossier.join(format!("{code}.json")),
            Value::Object(r).to_string(),
        )
        .await
        .is_err()
    {
        return refus(StatusCode::SERVICE_UNAVAILABLE, "storage");
    }
    eprintln!("Télémétrie : rapport {code}");
    Json(json!({ "id": code })).into_response()
}

/// `GET /api/telemetry/report/{code}` (jeton)
pub async fn lire_rapport(
    State(state): State<Arc<AppState>>,
    entetes: HeaderMap,
    Path(code): Path<String>,
) -> Response {
    let t = &state.telemetrie;
    if !t.lecture_autorisee(&entetes) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let code = code.to_ascii_uppercase();
    if !code_valide(&code) {
        return StatusCode::NOT_FOUND.into_response();
    }
    match tokio::fs::read(t.dossier.join("reports").join(format!("{code}.json"))).await {
        Ok(octets) => ([(header::CONTENT_TYPE, "application/json")], octets).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `GET /api/telemetry/summary` (jeton) : chiffres des 30 derniers jours.
pub async fn resume(State(state): State<Arc<AppState>>, entetes: HeaderMap) -> Response {
    let t = state.telemetrie.clone();
    if !t.lecture_autorisee(&entetes) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let resultat = tokio::task::spawn_blocking(move || compter(&t.dossier, 30)).await;
    match resultat {
        Ok(v) => Json(v).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn compter(dossier: &std::path::Path, jours: i64) -> Value {
    use std::collections::{BTreeMap, HashSet};
    let aujourd_hui = OffsetDateTime::now_utc().date();
    let mut par_jour: BTreeMap<String, Value> = BTreeMap::new();
    let mut evenements: BTreeMap<String, u64> = BTreeMap::new();
    let mut versions: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    let mut plantages: BTreeMap<String, u64> = BTreeMap::new();
    let mut parties: BTreeMap<String, u64> = BTreeMap::new();
    let mut installations: HashSet<String> = HashSet::new();
    for i in 0..jours {
        let Some(d) = aujourd_hui.checked_sub(time::Duration::days(i)) else { break };
        let nom = d
            .format(format_description!("[year]-[month]-[day]"))
            .unwrap_or_default();
        let Ok(texte) = std::fs::read_to_string(dossier.join(format!("events-{nom}.jsonl"))) else {
            continue;
        };
        let mut ids = HashSet::new();
        let mut n = 0u64;
        for ligne in texte.lines() {
            let Ok(v) = serde_json::from_str::<Value>(ligne) else { continue };
            n += 1;
            let id = v["id"].as_str().unwrap_or("").to_string();
            let ev = v["event"].as_str().unwrap_or("?").to_string();
            *evenements.entry(ev.clone()).or_default() += 1;
            if let Some(ver) = v["version"].as_str() {
                versions.entry(ver.to_string()).or_default().insert(id.clone());
            }
            let loader = v["data"]["loader"].as_str().unwrap_or("?").to_string();
            if ev == "game_start" {
                *parties.entry(loader.clone()).or_default() += 1;
            }
            if ev == "game_end" && v["data"]["crashed"].as_bool() == Some(true) {
                *plantages.entry(loader).or_default() += 1;
            }
            installations.insert(id.clone());
            ids.insert(id);
        }
        par_jour.insert(nom, json!({ "events": n, "installs": ids.len() }));
    }
    let rapports = std::fs::read_dir(dossier.join("reports"))
        .map(|l| l.flatten().count())
        .unwrap_or(0);
    json!({
        "days": jours,
        "installs": installations.len(),
        "per_day": par_jour,
        "events": evenements,
        "launcher_versions": versions.into_iter().map(|(k, v)| (k, v.len())).collect::<BTreeMap<_, _>>(),
        "games_by_loader": parties,
        "crashes_by_loader": plantages,
        "reports": rapports,
    })
}

/// Chiffres d'un jour (résumé Discord).
pub fn chiffres_du_jour(dossier: &std::path::Path, jour: time::Date) -> Value {
    use std::collections::{BTreeMap, HashSet};
    let nom = jour
        .format(format_description!("[year]-[month]-[day]"))
        .unwrap_or_default();
    let texte = std::fs::read_to_string(dossier.join(format!("events-{nom}.jsonl"))).unwrap_or_default();
    let mut ids = HashSet::new();
    let mut versions: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    let mut parties: BTreeMap<String, u64> = BTreeMap::new();
    let mut plantages: BTreeMap<String, u64> = BTreeMap::new();
    let (mut n_parties, mut n_plantages, mut n_maj) = (0u64, 0u64, 0u64);
    for ligne in texte.lines() {
        let Ok(v) = serde_json::from_str::<Value>(ligne) else { continue };
        let id = v["id"].as_str().unwrap_or("").to_string();
        if let Some(ver) = v["version"].as_str() {
            versions.entry(ver.to_string()).or_default().insert(id.clone());
        }
        let loader = v["data"]["loader"].as_str().unwrap_or("?").to_string();
        match v["event"].as_str().unwrap_or("") {
            "game_start" => {
                n_parties += 1;
                *parties.entry(loader).or_default() += 1;
            }
            "game_end" if v["data"]["crashed"].as_bool() == Some(true) => {
                n_plantages += 1;
                *plantages.entry(loader).or_default() += 1;
            }
            "update" => n_maj += 1,
            _ => {}
        }
        ids.insert(id);
    }
    // rapports reçus ce jour-là
    let rapports = std::fs::read_dir(dossier.join("reports"))
        .map(|l| {
            l.flatten()
                .filter(|e| {
                    e.metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .map(|m| OffsetDateTime::from(m).date() == jour)
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0);
    json!({
        "day": nom,
        "installs": ids.len(),
        "games": n_parties,
        "crashes": n_plantages,
        "updates": n_maj,
        "reports": rapports,
        "games_by_loader": parties,
        "crashes_by_loader": plantages,
        "launcher_versions": versions.into_iter().map(|(k, v)| (k, v.len())).collect::<BTreeMap<_, _>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes() {
        let c = code_rapport();
        assert!(code_valide(&c), "{c}");
        assert!(!code_valide("MR-../../x"));
        assert_ne!(code_rapport(), code_rapport());
    }

    #[test]
    fn debit() {
        let t = Telemetrie::new();
        for _ in 0..MAX_RAPPORTS {
            assert!(t.autorise("1.2.3.4", true));
        }
        assert!(!t.autorise("1.2.3.4", true));
        assert!(t.autorise("5.6.7.8", true));
        assert!(t.autorise("1.2.3.4", false));
    }

    #[test]
    fn jeton() {
        assert!(egal_constant(b"abc", b"abc"));
        assert!(!egal_constant(b"abc", b"abd"));
        assert!(!egal_constant(b"abc", b"abcd"));
    }
}
