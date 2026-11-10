//! Seed viewer (`/tools/seed`) : biomes et structures d'un monde Minecraft, calculés par
//! cubiomes (MIT, `vendor/cubiomes`), le même algorithme que le jeu.
//!
//! La carte est découpée en tuiles de `TUILE` × `TUILE` cases. Le navigateur les demande une par
//! une et les colore lui-même avec la palette, ce qui lui permet aussi d'afficher le biome sous
//! la souris sans redemander au serveur.
//!
//! - `GET /api/seed/info?seed=…&v=…` : seed (texte → nombre, comme le jeu) et point d'apparition.
//! - `GET /api/seed/biomes?v=…` : nom et couleur de chaque biome de la version.
//! - `GET /api/seed/tile?seed=…&v=…&dim=…&scale=…&x=…&z=…` : une tuile, un octet par case
//!   (identifiant du biome), en ligne par ligne.
//! - `GET /api/seed/structures?seed=…&v=…&dim=…&x=…&z=…` : structures d'une région de
//!   `REGION` × `REGION` blocs.
//!
//! Les calculs prennent du processeur : ils passent par un nombre limité de fils
//! (la moitié des cœurs, 4 au plus), les résultats sont gardés en cache, et le débit est limité par adresse.

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::ffi::{CStr, c_char};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::AppState;

unsafe extern "C" {
    fn misty_biomes(mc: i32, dim: i32, seed: i64, scale: i32, x: i32, z: i32, sx: i32, sz: i32, sortie: *mut i32) -> i32;
    fn misty_couleurs(sortie: *mut u8);
    fn misty_nom_biome(mc: i32, id: i32) -> *const c_char;
    fn misty_spawn(mc: i32, seed: i64, x: *mut i32, z: *mut i32);
    fn misty_structures(mc: i32, dim: i32, seed: i64, type_: i32, x0: i32, z0: i32, x1: i32, z1: i32, sortie: *mut i32, max: i32) -> i32;
}

/// Cases par côté d'une tuile.
pub const TUILE: i32 = 128;
/// Blocs par case possibles (zoom), du plus proche au plus loin.
pub const ECHELLES: [i32; 4] = [4, 16, 64, 256];
/// Blocs par côté d'une région de structures.
pub const REGION: i32 = 2048;
/// Au-delà, le monde n'existe plus (bordure du monde).
const LIMITE_MONDE: i64 = 30_000_000;
/// Requêtes de calcul par adresse et par minute (une carte plein écran demande ~30 tuiles).
const MAX_PAR_MINUTE: u32 = 1500;
/// Tuiles et régions gardées en mémoire (une tuile fait 16 Ko).
const MAX_CACHE: usize = 2048;

/// Versions proposées : (libellé, identifiant cubiomes), la plus récente d'abord.
pub const VERSIONS: &[(&str, i32)] = &[
    ("1.21.4+", 28),
    ("1.21.2 – 1.21.3", 27),
    ("1.21 – 1.21.1", 26),
    ("1.20", 25),
    ("1.19.3 – 1.19.4", 24),
    ("1.19 – 1.19.2", 23),
    ("1.18", 22),
    ("1.17", 21),
    ("1.16.2 – 1.16.5", 20),
    ("1.16.1", 19),
    ("1.15", 18),
    ("1.14", 17),
    ("1.13", 16),
    ("1.12", 15),
    ("1.11", 14),
    ("1.10", 13),
    ("1.9", 12),
    ("1.8", 11),
    ("1.7", 10),
    ("1.6", 9),
    ("1.5", 8),
    ("1.4", 7),
    ("1.3", 6),
    ("1.2", 5),
    ("1.1", 4),
    ("1.0", 3),
];

/// Le Nether a ses biomes depuis la 1.16 ; avant, cubiomes ne le génère pas.
const NETHER_DEPUIS: i32 = 19;
/// L'End a ses biomes (îles) depuis la 1.9.
const END_DEPUIS: i32 = 12;

/// Une sorte de structure : (clé, dimension, types cubiomes regroupés).
pub const STRUCTURES: &[(&str, i32, &[i32])] = &[
    ("village", 0, &[5]),
    ("outpost", 0, &[10]),
    ("mansion", 0, &[9]),
    ("monument", 0, &[8]),
    ("temple", 0, &[1, 2, 3, 4]),
    ("ruined_portal", 0, &[11]),
    ("ancient_city", 0, &[13]),
    ("trial_chambers", 0, &[24]),
    ("fortress", -1, &[18]),
    ("bastion", -1, &[19]),
    ("ruined_portal", -1, &[12]),
    ("end_city", 1, &[20]),
];

pub fn version_valide(mc: i32) -> bool {
    VERSIONS.iter().any(|(_, v)| *v == mc)
}

pub fn dimension_valide(mc: i32, dim: i32) -> bool {
    match dim {
        0 => true,
        -1 => mc >= NETHER_DEPUIS,
        1 => mc >= END_DEPUIS,
        _ => false,
    }
}

/// Seed saisie : un nombre, ou un texte (le jeu en fait un nombre avec `hashCode` de Java).
/// Vide : au hasard.
pub fn lire_seed(texte: &str) -> i64 {
    let t = texte.trim();
    if t.is_empty() {
        return rand::random::<i64>();
    }
    if let Ok(n) = t.parse::<i64>() {
        return n;
    }
    let mut h: i32 = 0;
    for u in t.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(u as i32);
    }
    h as i64
}

/// Biomes d'une tuile : `TUILE` × `TUILE` octets (identifiant du biome), ligne par ligne.
pub fn tuile(mc: i32, dim: i32, seed: i64, echelle: i32, tx: i32, tz: i32) -> Option<Vec<u8>> {
    let mut ids = vec![0i32; (TUILE * TUILE) as usize];
    // SAFETY : le tampon contient TUILE * TUILE entiers, la taille demandée
    let erreur = unsafe { misty_biomes(mc, dim, seed, echelle, tx * TUILE, tz * TUILE, TUILE, TUILE, ids.as_mut_ptr()) };
    (erreur == 0).then(|| ids.iter().map(|&id| id.clamp(0, 255) as u8).collect())
}

/// Structures de la région (rx, rz) : (indice dans `STRUCTURES`, x, z).
pub fn structures(mc: i32, dim: i32, seed: i64, rx: i32, rz: i32) -> Vec<(usize, i32, i32)> {
    let (x0, z0) = (rx * REGION, rz * REGION);
    let (x1, z1) = (x0 + REGION - 1, z0 + REGION - 1);
    let mut trouvees = Vec::new();
    for (i, (_, d, types)) in STRUCTURES.iter().enumerate() {
        if *d != dim {
            continue;
        }
        for &t in *types {
            let mut sortie = vec![0i32; 2 * 256];
            // SAFETY : la sortie peut contenir 256 positions
            let n = unsafe { misty_structures(mc, dim, seed, t, x0, z0, x1, z1, sortie.as_mut_ptr(), 256) };
            for k in 0..n.max(0) as usize {
                trouvees.push((i, sortie[k * 2], sortie[k * 2 + 1]));
            }
        }
    }
    trouvees
}

pub fn spawn(mc: i32, seed: i64) -> (i32, i32) {
    let (mut x, mut z) = (0, 0);
    // SAFETY : les pointeurs désignent des entiers valides
    unsafe { misty_spawn(mc, seed, &mut x, &mut z) };
    (x, z)
}

/// Nom et couleur de chaque biome connu de cette version : `{ "<id>": ["nom", "#rrggbb"] }`.
pub fn biomes(mc: i32) -> Value {
    let mut brut = vec![0u8; 256 * 3];
    // SAFETY : le tampon fait bien 256 * 3 octets
    unsafe { misty_couleurs(brut.as_mut_ptr()) };
    let mut m = serde_json::Map::new();
    for id in 0..256 {
        // SAFETY : misty_nom_biome renvoie une chaîne C statique, ou NULL
        let nom = unsafe {
            let p = misty_nom_biome(mc, id);
            if p.is_null() { continue } else { CStr::from_ptr(p).to_string_lossy().into_owned() }
        };
        let c = &brut[id as usize * 3..id as usize * 3 + 3];
        m.insert(id.to_string(), json!([nom, format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])]));
    }
    Value::Object(m)
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Cle {
    Tuile { mc: i32, dim: i32, seed: i64, echelle: i32, x: i32, z: i32 },
    Region { mc: i32, dim: i32, seed: i64, x: i32, z: i32 },
}

pub struct SeedMap {
    calculs: tokio::sync::Semaphore,
    cache: Mutex<HashMap<Cle, Arc<Vec<u8>>>>,
    debits: Mutex<HashMap<String, (Instant, u32)>>,
}

impl SeedMap {
    pub fn new() -> Self {
        // calculs cubiomes en même temps (le reste attend son tour) : le site doit rester rapide
        let coeurs = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
        Self { calculs: tokio::sync::Semaphore::new((coeurs / 2).clamp(1, 4)), cache: Mutex::default(), debits: Mutex::default() }
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

    /// Résultat en cache, ou calculé (hors du fil async, à tour de rôle).
    async fn obtenir(&self, cle: Cle, calcul: impl FnOnce() -> Option<Vec<u8>> + Send + 'static) -> Option<Arc<Vec<u8>>> {
        if let Some(v) = self.cache.lock().unwrap().get(&cle) {
            return Some(v.clone());
        }
        let _permis = self.calculs.acquire().await.ok()?;
        // calculée pendant l'attente par une autre requête ?
        if let Some(v) = self.cache.lock().unwrap().get(&cle) {
            return Some(v.clone());
        }
        let v = Arc::new(tokio::task::spawn_blocking(calcul).await.ok()??);
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= MAX_CACHE {
            cache.clear();
        }
        cache.insert(cle, v.clone());
        Some(v)
    }
}

#[derive(Deserialize)]
pub struct Params {
    seed: Option<String>,
    v: Option<i32>,
    dim: Option<i32>,
    scale: Option<i32>,
    x: Option<i32>,
    z: Option<i32>,
}

fn refus(code: StatusCode, message: &str) -> Response {
    (code, Json(json!({ "error": message }))).into_response()
}

/// Les réponses ne dépendent que des paramètres : le navigateur peut les garder longtemps.
fn immuable(mut res: Response) -> Response {
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=604800, immutable"));
    res
}

/// Version et seed (numérique) communes à toutes les routes.
fn base(p: &Params) -> Result<(i32, i64), Response> {
    let mc = p.v.filter(|v| version_valide(*v)).ok_or_else(|| refus(StatusCode::BAD_REQUEST, "bad version"))?;
    let seed = p.seed.as_deref().and_then(|s| s.trim().parse::<i64>().ok()).ok_or_else(|| refus(StatusCode::BAD_REQUEST, "bad seed"))?;
    Ok((mc, seed))
}

/// `GET /api/seed/info?seed=<texte>&v=<version>`
pub async fn info(Query(p): Query<Params>) -> Response {
    let Some(mc) = p.v.filter(|v| version_valide(*v)) else {
        return refus(StatusCode::BAD_REQUEST, "bad version");
    };
    let texte = p.seed.unwrap_or_default();
    if texte.chars().count() > 64 {
        return refus(StatusCode::BAD_REQUEST, "seed too long");
    }
    let seed = lire_seed(&texte);
    let Ok((x, z)) = tokio::task::spawn_blocking(move || spawn(mc, seed)).await else {
        return refus(StatusCode::INTERNAL_SERVER_ERROR, "spawn");
    };
    // la seed en texte : un i64 ne tient pas dans un nombre JavaScript
    Json(json!({ "seed": seed.to_string(), "spawn": { "x": x, "z": z } })).into_response()
}

/// `GET /api/seed/biomes?v=<version>`
pub async fn liste_biomes(Query(p): Query<Params>) -> Response {
    let Some(mc) = p.v.filter(|v| version_valide(*v)) else {
        return refus(StatusCode::BAD_REQUEST, "bad version");
    };
    immuable(Json(biomes(mc)).into_response())
}

/// `GET /api/seed/tile?seed=&v=&dim=&scale=&x=&z=` : `TUILE`² octets.
pub async fn tile(State(state): State<Arc<AppState>>, entetes: HeaderMap, Query(p): Query<Params>) -> Response {
    let (mc, seed) = match base(&p) {
        Ok(b) => b,
        Err(r) => return r,
    };
    let dim = p.dim.unwrap_or(0);
    let echelle = p.scale.unwrap_or(16);
    let (Some(x), Some(z)) = (p.x, p.z) else { return refus(StatusCode::BAD_REQUEST, "bad tile") };
    let blocs = (TUILE * echelle) as i64;
    let dans_le_monde = |t: i32| (t as i64 * blocs).abs() <= LIMITE_MONDE;
    if !dimension_valide(mc, dim) || !ECHELLES.contains(&echelle) || !dans_le_monde(x) || !dans_le_monde(z) {
        return refus(StatusCode::BAD_REQUEST, "bad tile");
    }
    let s = &state.seedmap;
    if !s.autorise(&crate::presence::adresse(&entetes)) {
        return refus(StatusCode::TOO_MANY_REQUESTS, "slow down");
    }
    let cle = Cle::Tuile { mc, dim, seed, echelle, x, z };
    match s.obtenir(cle, move || tuile(mc, dim, seed, echelle, x, z)).await {
        Some(octets) => {
            let mut res = (*octets).clone().into_response();
            // un type « texte » pour que la réponse soit compressée (16 Ko → 1 à 3 Ko)
            res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/x-misty-biomes"));
            immuable(res)
        }
        None => refus(StatusCode::INTERNAL_SERVER_ERROR, "generation failed"),
    }
}

/// `GET /api/seed/structures?seed=&v=&dim=&x=&z=` (x, z : région de `REGION` blocs)
/// → `[["village", x, z], …]`
pub async fn structures_region(State(state): State<Arc<AppState>>, entetes: HeaderMap, Query(p): Query<Params>) -> Response {
    let (mc, seed) = match base(&p) {
        Ok(b) => b,
        Err(r) => return r,
    };
    let dim = p.dim.unwrap_or(0);
    let (Some(x), Some(z)) = (p.x, p.z) else { return refus(StatusCode::BAD_REQUEST, "bad region") };
    let dans_le_monde = |r: i32| (r as i64 * REGION as i64).abs() <= LIMITE_MONDE;
    if !dimension_valide(mc, dim) || !dans_le_monde(x) || !dans_le_monde(z) {
        return refus(StatusCode::BAD_REQUEST, "bad region");
    }
    let s = &state.seedmap;
    if !s.autorise(&crate::presence::adresse(&entetes)) {
        return refus(StatusCode::TOO_MANY_REQUESTS, "slow down");
    }
    let cle = Cle::Region { mc, dim, seed, x, z };
    let calcul = move || {
        let liste: Vec<Value> = structures(mc, dim, seed, x, z).into_iter().map(|(i, bx, bz)| json!([STRUCTURES[i].0, bx, bz])).collect();
        Some(Value::Array(liste).to_string().into_bytes())
    };
    match s.obtenir(cle, calcul).await {
        Some(octets) => {
            let mut res = (*octets).clone().into_response();
            res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
            immuable(res)
        }
        None => refus(StatusCode::INTERNAL_SERVER_ERROR, "generation failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_texte_comme_java() {
        assert_eq!(lire_seed("12345"), 12345);
        assert_eq!(lire_seed("-5"), -5);
        assert_eq!(lire_seed("Hello"), 69609650); // "Hello".hashCode() en Java
    }

    #[test]
    fn toutes_les_versions_et_dimensions_se_generent() {
        for &(nom, mc) in VERSIONS {
            for dim in [0, -1, 1] {
                if !dimension_valide(mc, dim) {
                    continue;
                }
                for echelle in ECHELLES {
                    let t = tuile(mc, dim, 42, echelle, -1, 0).unwrap_or_else(|| panic!("{nom} dim {dim} échelle {echelle}"));
                    assert_eq!(t.len(), (TUILE * TUILE) as usize);
                }
            }
        }
    }

    #[test]
    fn biomes_connus_et_nommes() {
        let b = biomes(28);
        let t = tuile(28, 0, 1, 16, 0, 0).unwrap();
        for id in t {
            assert!(b.get(id.to_string()).is_some(), "biome {id} sans nom");
        }
        assert_eq!(b["1"][0], "plains");
    }

    #[test]
    fn des_villages_quelque_part() {
        let n: usize = (-2..2).flat_map(|x| (-2..2).map(move |z| (x, z))).map(|(x, z)| structures(28, 0, 1, x, z).iter().filter(|s| STRUCTURES[s.0].0 == "village").count()).sum();
        assert!(n > 0);
        let (sx, sz) = spawn(28, 1);
        assert!(sx.abs() < 5000 && sz.abs() < 5000);
    }

    /// Temps de calcul : `cargo test --release mesure -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn mesure() {
        for mc in [28, 22, 15] {
            for echelle in ECHELLES {
                let t = Instant::now();
                tuile(mc, 0, 7, echelle, 3, -2);
                println!("mc {mc} échelle {echelle} : {:?}", t.elapsed());
            }
            let t = Instant::now();
            structures(mc, 0, 7, 1, 1);
            println!("mc {mc} structures : {:?}", t.elapsed());
        }
    }
}
