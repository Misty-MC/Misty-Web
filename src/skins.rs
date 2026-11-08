//! Relais des skins Minecraft pour l'aperçu 3D de la boutique.
//!
//! Le navigateur ne peut pas lire en WebGL une image d'un autre domaine sans en-têtes CORS :
//! le serveur récupère donc le skin (par défaut sur mc-heads.net) et le renvoie depuis le site,
//! avec un petit cache en mémoire.

use axum::body::Bytes;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const TTL: Duration = Duration::from_secs(600);
const MAX_ENTRIES: usize = 256;
/// Un skin fait quelques Ko ; au-delà, ce n'est pas un skin.
const MAX_BYTES: usize = 512 * 1024;

pub struct Skins {
    base: String,
    cache: Mutex<HashMap<String, (Instant, Bytes)>>,
}

impl Skins {
    pub fn new(base: String) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// PNG du skin de `name` (pseudo déjà validé), ou `None` si le service ne répond pas.
    pub async fn get(&self, http: &reqwest::Client, name: &str) -> Option<Bytes> {
        let key = name.to_ascii_lowercase();
        {
            let cache = self.cache.lock().await;
            if let Some((at, png)) = cache.get(&key)
                && at.elapsed() < TTL
            {
                return Some(png.clone());
            }
        }
        let res = http
            .get(format!("{}/skin/{name}", self.base))
            .send()
            .await
            .ok()?;
        if !res.status().is_success() {
            return None;
        }
        let png = res.bytes().await.ok()?;
        if png.len() > MAX_BYTES || !png.starts_with(b"\x89PNG") {
            return None;
        }
        let mut cache = self.cache.lock().await;
        if cache.len() >= MAX_ENTRIES {
            cache.retain(|_, (at, _)| at.elapsed() < TTL);
            if cache.len() >= MAX_ENTRIES {
                cache.clear();
            }
        }
        cache.insert(key, (Instant::now(), png.clone()));
        Some(png)
    }
}
