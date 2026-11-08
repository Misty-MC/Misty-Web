//! Profils Minecraft pour le studio de la boutique : skin, cape, modèle de bras, UUID.
//!
//! Le navigateur ne peut pas lire en WebGL une image d'un autre domaine sans en-têtes CORS :
//! le serveur récupère donc le skin et la cape et les renvoie depuis le site.
//!
//! Source principale : l'API officielle de Mojang (pseudo → UUID → textures). Si Mojang ne
//! répond pas (limite de requêtes, panne…), on se rabat sur `SKIN_API` (mc-heads.net par
//! défaut), qui ne donne que le skin. Les réponses sont gardées en cache quelques minutes.

use axum::body::Bytes;
use base64::Engine;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const TTL: Duration = Duration::from_secs(600);
const TTL_MISSING: Duration = Duration::from_secs(120);
/// Délai minimal entre deux rechargements forcés d'un même joueur (bouton « Actualiser »).
const MIN_REFRESH: Duration = Duration::from_secs(20);
const MAX_ENTRIES: usize = 256;
/// Un skin ou une cape fait quelques Ko ; au-delà, ce n'est pas une texture.
const MAX_BYTES: usize = 512 * 1024;

const MOJANG_PROFILE: &str = "https://api.mojang.com/users/profiles/minecraft";
const MOJANG_SESSION: &str = "https://sessionserver.mojang.com/session/minecraft/profile";

pub struct Profile {
    /// Pseudo avec la bonne casse (celui de Mojang si on l'a).
    pub name: String,
    /// UUID sans tirets, si Mojang a répondu.
    pub uuid: Option<String>,
    /// Bras fins (modèle « Alex »).
    pub slim: bool,
    pub skin: Bytes,
    pub cape: Option<Bytes>,
}

#[derive(Clone)]
pub enum Lookup {
    Found(Arc<Profile>),
    /// Ce pseudo n'existe pas chez Mojang.
    NotFound,
    /// Mojang et le service de secours sont injoignables.
    Unavailable,
}

pub struct Skins {
    fallback: String,
    cache: Mutex<HashMap<String, (Instant, Lookup)>>,
}

#[derive(Deserialize)]
struct MojangId {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct SessionProfile {
    properties: Vec<SessionProperty>,
}

#[derive(Deserialize)]
struct SessionProperty {
    name: String,
    value: String,
}

#[derive(Deserialize)]
struct TexturesPayload {
    textures: Textures,
}

#[derive(Deserialize)]
struct Textures {
    #[serde(rename = "SKIN")]
    skin: Option<Texture>,
    #[serde(rename = "CAPE")]
    cape: Option<Texture>,
}

#[derive(Deserialize)]
struct Texture {
    url: String,
    metadata: Option<TextureMeta>,
}

#[derive(Deserialize)]
struct TextureMeta {
    model: Option<String>,
}

/// Erreurs internes : « n'existe pas » est distingué de « injoignable ».
enum Fetch {
    NotFound,
    Failed,
}

async fn png(http: &reqwest::Client, url: &str) -> Option<Bytes> {
    // Mojang renvoie des URL en http:// ; textures.minecraft.net sert aussi en https.
    let url = url.replacen("http://", "https://", 1);
    let res = http.get(url).send().await.ok()?.error_for_status().ok()?;
    let bytes = res.bytes().await.ok()?;
    (bytes.len() <= MAX_BYTES && bytes.starts_with(b"\x89PNG")).then_some(bytes)
}

impl Skins {
    pub fn new(fallback: String) -> Self {
        Self {
            fallback: fallback.trim_end_matches('/').to_string(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Profil de `name` (pseudo déjà validé : 3 à 16 caractères `[A-Za-z0-9_]`).
    /// `fresh` ignore le cache (au plus une fois toutes les 20 s par joueur), pour voir tout de
    /// suite un skin ou une cape qui vient de changer.
    pub async fn lookup(&self, http: &reqwest::Client, name: &str, fresh: bool) -> Lookup {
        let key = name.to_ascii_lowercase();
        if let Some((at, found)) = self.cache.lock().await.get(&key) {
            let ttl = if matches!(found, Lookup::Found(_)) {
                TTL
            } else {
                TTL_MISSING
            };
            let ttl = if fresh { MIN_REFRESH } else { ttl };
            if at.elapsed() < ttl {
                return found.clone();
            }
        }
        let result = match self.fetch_mojang(http, name).await {
            Ok(p) => Lookup::Found(Arc::new(p)),
            Err(Fetch::NotFound) => Lookup::NotFound,
            Err(Fetch::Failed) => match self.fetch_fallback(http, name).await {
                Some(p) => Lookup::Found(Arc::new(p)),
                None => Lookup::Unavailable,
            },
        };
        let mut cache = self.cache.lock().await;
        if cache.len() >= MAX_ENTRIES {
            cache.retain(|_, (at, _)| at.elapsed() < TTL);
            if cache.len() >= MAX_ENTRIES {
                cache.clear();
            }
        }
        cache.insert(key, (Instant::now(), result.clone()));
        result
    }

    async fn fetch_mojang(&self, http: &reqwest::Client, name: &str) -> Result<Profile, Fetch> {
        let res = http
            .get(format!("{MOJANG_PROFILE}/{name}"))
            .send()
            .await
            .map_err(|_| Fetch::Failed)?;
        match res.status().as_u16() {
            200 => {}
            204 | 404 => return Err(Fetch::NotFound),
            _ => return Err(Fetch::Failed),
        }
        let id: MojangId = res.json().await.map_err(|_| Fetch::Failed)?;
        let session: SessionProfile = http
            .get(format!("{MOJANG_SESSION}/{}", id.id))
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|_| Fetch::Failed)?
            .json()
            .await
            .map_err(|_| Fetch::Failed)?;
        let encoded = session
            .properties
            .iter()
            .find(|p| p.name == "textures")
            .ok_or(Fetch::Failed)?;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded.value)
            .map_err(|_| Fetch::Failed)?;
        let payload: TexturesPayload =
            serde_json::from_slice(&decoded).map_err(|_| Fetch::Failed)?;

        // Sans skin personnalisé, Mojang n'envoie pas de texture : on prend celle du service de secours.
        let (skin, slim) = match &payload.textures.skin {
            Some(t) => (
                png(http, &t.url).await.ok_or(Fetch::Failed)?,
                t.metadata.as_ref().and_then(|m| m.model.as_deref()) == Some("slim"),
            ),
            None => (
                self.fallback_skin(http, name).await.ok_or(Fetch::Failed)?,
                false,
            ),
        };
        let cape = match &payload.textures.cape {
            Some(t) => png(http, &t.url).await,
            None => None,
        };
        Ok(Profile {
            name: id.name,
            uuid: Some(id.id),
            slim,
            skin,
            cape,
        })
    }

    async fn fallback_skin(&self, http: &reqwest::Client, name: &str) -> Option<Bytes> {
        png(http, &format!("{}/skin/{name}", self.fallback)).await
    }

    async fn fetch_fallback(&self, http: &reqwest::Client, name: &str) -> Option<Profile> {
        let skin = self.fallback_skin(http, name).await?;
        Some(Profile {
            name: name.to_string(),
            uuid: None,
            slim: false,
            skin,
            cape: None,
        })
    }
}

/// UUID au format affiché (avec tirets).
pub fn dashed_uuid(id: &str) -> String {
    if id.len() != 32 {
        return id.to_string();
    }
    format!(
        "{}-{}-{}-{}-{}",
        &id[0..8],
        &id[8..12],
        &id[12..16],
        &id[16..20],
        &id[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_dashes() {
        assert_eq!(
            dashed_uuid("1439f2f7a58a4b338949faa4710911c1"),
            "1439f2f7-a58a-4b33-8949-faa4710911c1"
        );
        assert_eq!(dashed_uuid("court"), "court");
    }
}
