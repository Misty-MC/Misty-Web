//! Joueurs en ligne sur le launcher.
//!
//! Chaque launcher ouvert envoie un signe de vie toutes les 60 secondes (`POST /api/presence`)
//! avec un identifiant tiré au hasard à chaque démarrage (rien qui reste d'une session à l'autre)
//! et s'il a une partie en cours. `GET /api/online` compte les launchers vus ces 3 dernières
//! minutes. Tout reste en mémoire : rien n'est écrit, aucune adresse IP n'est gardée.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::AppState;

/// Un launcher compte comme en ligne tant que son dernier signe date de moins de 3 minutes.
const EN_LIGNE: Duration = Duration::from_secs(180);
/// Signes de vie acceptés par adresse et par minute (plusieurs launchers derrière la même box).
const MAX_PAR_MINUTE: u32 = 60;
/// Au-delà, les sessions les plus anciennes sont oubliées (protection mémoire).
const MAX_SESSIONS: usize = 200_000;

struct Session {
    vu: Instant,
    en_jeu: bool,
}

#[derive(Default)]
pub struct Presence {
    sessions: Mutex<HashMap<String, Session>>,
    /// adresse → (début de la minute, compte), seulement pour limiter le débit
    debits: Mutex<HashMap<String, (Instant, u32)>>,
    /// Record du nombre de launchers en ligne depuis le démarrage du site
    record: Mutex<(usize, Option<time::OffsetDateTime>)>,
}

impl Presence {
    pub fn new() -> Self {
        Self::default()
    }

    fn autorise(&self, adresse: &str) -> bool {
        let mut d = self.debits.lock().unwrap();
        if d.len() > 50_000 {
            d.retain(|_, (debut, _)| debut.elapsed() < Duration::from_secs(60));
        }
        let e = d.entry(adresse.to_string()).or_insert((Instant::now(), 0));
        if e.0.elapsed() >= Duration::from_secs(60) {
            *e = (Instant::now(), 0);
        }
        e.1 += 1;
        e.1 <= MAX_PAR_MINUTE
    }

    fn signe(&self, session: String, en_jeu: bool) {
        let mut s = self.sessions.lock().unwrap();
        if s.len() >= MAX_SESSIONS && !s.contains_key(&session) {
            s.retain(|_, x| x.vu.elapsed() < EN_LIGNE);
            if s.len() >= MAX_SESSIONS {
                return;
            }
        }
        s.insert(session, Session { vu: Instant::now(), en_jeu });
    }

    fn quitte(&self, session: &str) {
        self.sessions.lock().unwrap().remove(session);
    }

    /// (launchers ouverts, dont en partie)
    pub fn compte(&self) -> (usize, usize) {
        let mut s = self.sessions.lock().unwrap();
        s.retain(|_, x| x.vu.elapsed() < EN_LIGNE);
        let en_jeu = s.values().filter(|x| x.en_jeu).count();
        let n = s.len();
        drop(s);
        let mut r = self.record.lock().unwrap();
        if n > r.0 {
            *r = (n, Some(time::OffsetDateTime::now_utc()));
        }
        (n, en_jeu)
    }
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

/// `POST /api/presence` : `{ "session": "<aléatoire>", "playing": true, "bye": false }`.
/// `bye` : le launcher se ferme (il ne compte plus tout de suite).
pub async fn signe(State(state): State<Arc<AppState>>, entetes: HeaderMap, Json(corps): Json<Value>) -> Response {
    let p = &state.presence;
    if !p.autorise(&adresse(&entetes)) {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    let Some(session) = corps["session"]
        .as_str()
        .map(str::trim)
        .filter(|s| (8..=64).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
    else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "bad session" }))).into_response();
    };
    if corps["bye"].as_bool() == Some(true) {
        p.quitte(session);
    } else {
        p.signe(session.to_string(), corps["playing"].as_bool().unwrap_or(false));
    }
    StatusCode::NO_CONTENT.into_response()
}

/// `GET /api/online` : `{ "online": 42, "playing": 17, "record": 80, "recordAt": "…" }`.
/// Ouvert à tous (bot Discord, widgets, site) ; mis en cache 15 secondes.
pub async fn en_ligne(State(state): State<Arc<AppState>>) -> Response {
    let (n, en_jeu) = state.presence.compte();
    let (record, quand) = *state.presence.record.lock().unwrap();
    let quand = quand.and_then(|d| d.format(&time::format_description::well_known::Rfc3339).ok());
    let mut res = Json(json!({
        "online": n,
        "playing": en_jeu,
        "record": record,
        "recordAt": quand,
    }))
    .into_response();
    let h = res.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=15"));
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compte_et_depart() {
        let p = Presence::new();
        p.signe("aaaaaaaa-1".into(), false);
        p.signe("bbbbbbbb-2".into(), true);
        p.signe("bbbbbbbb-2".into(), true); // même launcher : compté une fois
        assert_eq!(p.compte(), (2, 1));
        p.quitte("aaaaaaaa-1");
        assert_eq!(p.compte(), (1, 1));
        assert_eq!(p.record.lock().unwrap().0, 2);
    }

    #[test]
    fn debit() {
        let p = Presence::new();
        for _ in 0..MAX_PAR_MINUTE {
            assert!(p.autorise("1.2.3.4"));
        }
        assert!(!p.autorise("1.2.3.4"));
        assert!(p.autorise("5.6.7.8"));
    }
}
