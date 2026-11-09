//! Vérification Discord : captcha pour les nouveaux membres des serveurs qui utilisent Misty Bot.
//!
//! 1. Dans Discord, le membre clique sur « Vérifier » : le bot lui donne un lien signé
//!    `/verify?t=v1.<serveur>.<membre>.<rôle>.<salon de logs ou 0>.<expiration>.<langue>.<signature>`
//!    (HMAC-SHA256 avec `VERIFY_SECRET`, le même secret que le bot ; valable 10 minutes).
//! 2. Le site vérifie la signature et affiche un captcha « puzzle » (voir `crate::puzzle`) : faire
//!    glisser une pièce jusqu'au trou de l'image.
//! 3. Réponse juste : le site donne le rôle lui-même via l'API Discord (`DISCORD_TOKEN`, le token
//!    du bot) et note la vérification dans le salon de logs. Le bot n'a pas besoin d'être joignable.
//!
//! Limites : 3 essais par image, 10 images par lien, et le geste doit ressembler à celui d'un humain.
//!
//! **Dans Discord (Activité)** : le bouton « Vérifier » ouvre le site *dans* Discord
//! (`https://<application>.discordsays.com`, relayé vers ce site). La page `/activity` utilise le
//! SDK Embedded App de Discord pour savoir qui est le membre (autorisation OAuth2 `identify`,
//! code échangé ici avec `DISCORD_CLIENT_SECRET`) et dans quel serveur il est. Le rôle et le salon
//! de logs de ce serveur viennent du bot, qui les envoie signés à `/api/verify/config`.

use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const ESSAIS_PAR_IMAGE: u32 = 3;
const IMAGES_PAR_LIEN: u32 = 10;
const VIE_IMAGE: Duration = Duration::from_secs(600);

/// Contenu d'un lien signé par le bot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claims {
    pub guild: u64,
    pub user: u64,
    pub role: u64,
    /// Rôle retiré une fois vérifié
    pub remove: Option<u64>,
    pub log: Option<u64>,
    pub expires: u64,
    /// `fr` ou `en`
    pub lang: String,
}

fn signer(secret: &str, payload: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC accepte toute clé");
    mac.update(payload.as_bytes());
    mac.finalize().into_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

/// Vérifie la signature et l'expiration d'un lien.
pub fn verifier_lien(secret: &str, token: &str, maintenant: u64) -> Option<Claims> {
    let (payload, sig) = token.rsplit_once('.')?;
    // comparaison en temps constant
    let attendu = signer(secret, payload);
    if attendu.len() != sig.len() || attendu.bytes().zip(sig.bytes()).fold(0u8, |a, (x, y)| a | (x ^ y)) != 0 {
        return None;
    }
    let p: Vec<&str> = payload.split('.').collect();
    // v1 : sans rôle à retirer ; v2 : avec (0 = aucun)
    let (g, u, r, rm, log, exp, lang) = match p.as_slice() {
        ["v1", g, u, r, log, exp, lang] => (g, u, r, &"0", log, exp, lang),
        ["v2", g, u, r, rm, log, exp, lang] => (g, u, r, rm, log, exp, lang),
        _ => return None,
    };
    let c = Claims {
        guild: g.parse().ok()?,
        user: u.parse().ok()?,
        role: r.parse().ok()?,
        remove: rm.parse().ok().filter(|r| *r != 0),
        log: log.parse().ok().filter(|l| *l != 0),
        expires: exp.parse().ok()?,
        lang: lang.to_string(),
    };
    (c.expires > maintenant).then_some(c)
}

pub fn maintenant() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

struct Defi {
    /// Graine de l'image (position du trou, paysage)
    graine: u64,
    token: String,
    cree: Instant,
    essais: u32,
}

/// Résultat d'une réponse au captcha.
#[derive(Debug, PartialEq, Eq)]
pub enum Reponse {
    Juste,
    Faux,
    /// Image inconnue, expirée ou trop d'essais : il faut une nouvelle image
    Perime,
}

pub struct Verification {
    secret: Option<String>,
    bot_token: Option<String>,
    /// `https://discord.com/api/v10` (modifiable pour les tests : `DISCORD_API`)
    api: String,
    http: reqwest::Client,
    defis: Mutex<HashMap<String, Defi>>,
    /// Images générées par lien
    par_lien: Mutex<HashMap<String, (u32, Instant)>>,
    /// Activité Discord : identifiant et secret OAuth2 de l'application
    client_id: Option<String>,
    client_secret: Option<String>,
    /// Config de vérification de chaque serveur, envoyée par le bot
    serveurs: Mutex<HashMap<String, ConfigServeur>>,
    fichier_serveurs: PathBuf,
}

/// Ce que le bot envoie pour un serveur (identifiants en texte : trop grands pour JavaScript).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigServeur {
    pub guild: String,
    pub enabled: bool,
    pub role: Option<String>,
    #[serde(default)]
    pub remove: Option<String>,
    pub log: Option<String>,
    pub lang: String,
    /// Nom du serveur Discord (affiché sur la page)
    #[serde(default)]
    pub name: Option<String>,
}

/// Corps `application/x-www-form-urlencoded`.
fn formulaire(champs: &[(&str, &str)]) -> String {
    let encoder = |v: &str| -> String {
        v.bytes()
            .map(|b| match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
                _ => format!("%{b:02X}"),
            })
            .collect()
    };
    champs.iter().map(|(k, v)| format!("{k}={}", encoder(v))).collect::<Vec<_>>().join("&")
}

/// L'identifiant de l'application se lit au début du token du bot (base64 de l'identifiant).
fn id_depuis_token(token: &str) -> Option<String> {
    use base64::Engine;
    let debut = token.split('.').next()?;
    let octets = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(debut.trim_end_matches('='))
        .ok()?;
    let id = String::from_utf8(octets).ok()?;
    id.bytes().all(|b| b.is_ascii_digit()).then_some(id)
}

impl Verification {
    pub fn new() -> Self {
        let lire = |k: &str| std::env::var(k).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let secret = lire("VERIFY_SECRET");
        let bot_token = lire("DISCORD_TOKEN");
        if secret.is_some() != bot_token.is_some() {
            eprintln!("Vérification Discord : il faut VERIFY_SECRET et DISCORD_TOKEN (un seul est défini)");
        }
        let http = reqwest::Client::builder()
            .user_agent("DiscordBot (https://misty-launcher.online, 1.0) misty-site")
            .timeout(Duration::from_secs(10))
            .build()
            .expect("client HTTP");
        let api = lire("DISCORD_API").unwrap_or_else(|| "https://discord.com/api/v10".into());
        let client_id = lire("DISCORD_CLIENT_ID").or_else(|| bot_token.as_deref().and_then(id_depuis_token));
        let client_secret = lire("DISCORD_CLIENT_SECRET");
        let fichier_serveurs = std::env::var_os("MISTY_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("misty-site-verify-guilds.json");
        let serveurs = std::fs::read(&fichier_serveurs)
            .ok()
            .and_then(|o| serde_json::from_slice(&o).ok())
            .unwrap_or_default();
        Self {
            secret,
            bot_token,
            api,
            http,
            defis: Mutex::default(),
            par_lien: Mutex::default(),
            client_id,
            client_secret,
            serveurs: Mutex::new(serveurs),
            fichier_serveurs,
        }
    }

    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    /// L'Activité Discord est utilisable (secret OAuth2 en plus du reste).
    pub fn activite_active(&self) -> bool {
        self.active() && self.client_id.is_some() && self.client_secret.is_some()
    }

    /// Config envoyée par le bot : enregistrée si la signature est bonne.
    pub fn recevoir_config(&self, corps: &[u8], signature: &str) -> bool {
        let Some(secret) = &self.secret else { return false };
        let attendu = signer(secret, &String::from_utf8_lossy(corps));
        if attendu.len() != signature.len() || attendu.bytes().zip(signature.bytes()).fold(0u8, |a, (x, y)| a | (x ^ y)) != 0 {
            return false;
        }
        let Ok(c) = serde_json::from_slice::<ConfigServeur>(corps) else { return false };
        let json = {
            let mut s = self.serveurs.lock().unwrap();
            s.insert(c.guild.clone(), c);
            serde_json::to_vec(&*s).ok()
        };
        if let Some(json) = json {
            let _ = std::fs::write(&self.fichier_serveurs, json);
        }
        true
    }

    pub fn config(&self, guild: &str) -> Option<ConfigServeur> {
        self.serveurs.lock().unwrap().get(guild).cloned()
    }

    /// Activité : échange le code OAuth2 donné par Discord contre l'identifiant du membre.
    pub async fn membre(&self, code: &str) -> Result<u64, String> {
        let (Some(id), Some(secret)) = (&self.client_id, &self.client_secret) else {
            return Err("DISCORD_CLIENT_SECRET manquant".into());
        };
        #[derive(Deserialize)]
        struct Jeton {
            access_token: String,
        }
        #[derive(Deserialize)]
        struct Moi {
            id: String,
        }
        let jeton: Jeton = self
            .http
            .post(format!("{}/oauth2/token", self.api))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(formulaire(&[("client_id", id), ("client_secret", secret), ("grant_type", "authorization_code"), ("code", code)]))
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        let moi: Moi = self
            .http
            .get(format!("{}/users/@me", self.api))
            .bearer_auth(&jeton.access_token)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        moi.id.parse().map_err(|_| "identifiant invalide".into())
    }

    /// Lien signé pour ce membre de ce serveur (comme ceux du bot), d'après la config reçue.
    pub fn lien_pour(&self, guild: &str, user: u64) -> Option<String> {
        let secret = self.secret.as_deref()?;
        let c = self.config(guild).filter(|c| c.enabled)?;
        let payload = format!(
            "v2.{}.{user}.{}.{}.{}.{}.{}",
            c.guild.parse::<u64>().ok()?,
            c.role.as_deref()?.parse::<u64>().ok()?,
            c.remove.as_deref().and_then(|r| r.parse::<u64>().ok()).unwrap_or(0),
            c.log.as_deref().and_then(|l| l.parse::<u64>().ok()).unwrap_or(0),
            maintenant() + 600,
            c.lang
        );
        Some(format!("{payload}.{}", signer(secret, &payload)))
    }

    pub fn active(&self) -> bool {
        self.secret.is_some() && self.bot_token.is_some()
    }

    pub fn lien(&self, token: &str) -> Option<Claims> {
        verifier_lien(self.secret.as_deref()?, token, maintenant())
    }

    /// Nouveau puzzle pour ce lien : (identifiant, hauteur de la pièce) ; `None` après trop de défis.
    pub fn nouveau_defi(&self, token: &str) -> Option<(String, u32)> {
        {
            let mut par_lien = self.par_lien.lock().unwrap();
            par_lien.retain(|_, (_, t)| t.elapsed() < VIE_IMAGE * 2);
            let n = par_lien.entry(token.to_string()).or_insert((0, Instant::now()));
            if n.0 >= IMAGES_PAR_LIEN {
                return None;
            }
            n.0 += 1;
        }
        let mut rng = rand::rng();
        let graine: u64 = rng.random();
        let id: String = (0..24).map(|_| format!("{:x}", rng.random_range(0..16u8))).collect();
        let mut defis = self.defis.lock().unwrap();
        defis.retain(|_, d| d.cree.elapsed() < VIE_IMAGE);
        if defis.len() > 20_000 {
            return None;
        }
        defis.insert(id.clone(), Defi { graine, token: token.to_string(), cree: Instant::now(), essais: 0 });
        Some((id, crate::puzzle::position(graine).1))
    }

    /// Image d'un défi (PNG) : le fond avec le trou, ou la pièce.
    pub fn image(&self, id: &str, piece: bool) -> Option<Vec<u8>> {
        let graine = {
            let defis = self.defis.lock().unwrap();
            defis.get(id).filter(|d| d.cree.elapsed() < VIE_IMAGE)?.graine
        };
        if piece { crate::puzzle::image_piece(graine) } else { crate::puzzle::image_fond(graine) }
    }

    /// Réponse : position finale de la pièce (pixels de l'image) et tracé du geste.
    pub fn repondre(&self, id: &str, token: &str, x: f32, trace: &[(f32, f32)]) -> Reponse {
        let mut defis = self.defis.lock().unwrap();
        let Some(d) = defis.get_mut(id).filter(|d| d.token == token && d.cree.elapsed() < VIE_IMAGE) else {
            return Reponse::Perime;
        };
        d.essais += 1;
        if crate::puzzle::bonne_position(d.graine, x) && crate::puzzle::geste_humain(trace) {
            defis.remove(id);
            Reponse::Juste
        } else if d.essais >= ESSAIS_PAR_IMAGE {
            defis.remove(id);
            Reponse::Perime
        } else {
            Reponse::Faux
        }
    }

    /// Donne le rôle au membre, puis note la vérification dans le salon de logs.
    pub async fn donner_role(&self, c: &Claims) -> Result<(), String> {
        let token = self.bot_token.as_deref().ok_or("DISCORD_TOKEN manquant")?;
        let url = format!("{}/guilds/{}/members/{}/roles/{}", self.api, c.guild, c.user, c.role);
        let r = self
            .http
            .put(url)
            .header("Authorization", format!("Bot {token}"))
            .header("X-Audit-Log-Reason", "Misty%20%3A%20captcha%20r%C3%A9ussi")
            .header("Content-Length", "0")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !r.status().is_success() {
            let code = r.status();
            let corps = r.text().await.unwrap_or_default();
            return Err(format!("Discord {code} : {}", corps.chars().take(200).collect::<String>()));
        }
        // rôle à retirer (facultatif) : une erreur n'empêche pas la vérification
        if let Some(rm) = c.remove {
            let url = format!("{}/guilds/{}/members/{}/roles/{rm}", self.api, c.guild, c.user);
            match self
                .http
                .delete(url)
                .header("Authorization", format!("Bot {token}"))
                .header("X-Audit-Log-Reason", "Misty%20%3A%20captcha%20r%C3%A9ussi")
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => {}
                Ok(r) => eprintln!("Vérification : rôle à retirer non retiré (serveur {}) : {}", c.guild, r.status()),
                Err(e) => eprintln!("Vérification : rôle à retirer non retiré (serveur {}) : {e}", c.guild),
            }
        }
        if let Some(log) = c.log {
            let (titre, comment) = if c.lang == "fr" {
                ("Membre vérifié", "captcha réussi")
            } else {
                ("Member verified", "captcha solved")
            };
            let message = serde_json::json!({
                "flags": 1 << 15,
                "allowed_mentions": { "parse": [] },
                "components": [{
                    "type": 17,
                    "accent_color": 0x4cc38a,
                    "components": [{ "type": 10, "content": format!("**{titre}** · <@{}> · {comment} · <t:{}:R>", c.user, maintenant()) }]
                }]
            });
            let _ = self
                .http
                .post(format!("{}/channels/{log}/messages", self.api))
                .header("Authorization", format!("Bot {token}"))
                .json(&message)
                .send()
                .await;
        }
        Ok(())
    }
}

impl Default for Verification {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liens_signes() {
        // même format et même secret que le bot (src/verify.rs de Misty-Bot)
        let payload = "v1.42.7.9.5.2000.en";
        let token = format!("{payload}.{}", signer("secret", payload));
        let c = verifier_lien("secret", &token, 1999).unwrap();
        assert_eq!((c.guild, c.user, c.role, c.log, c.lang.as_str()), (42, 7, 9, Some(5), "en"));
        assert!(verifier_lien("secret", &token, 2000).is_none(), "expiré");
        assert!(verifier_lien("autre", &token, 0).is_none(), "mauvais secret");
        assert!(verifier_lien("secret", &token.replace("v1.42.7", "v1.42.8"), 0).is_none(), "modifié");
        assert!(verifier_lien("secret", "n'importe quoi", 0).is_none());
        // v2 : avec le rôle à retirer (même vecteur que le test du bot)
        let payload = "v2.42.7.9.4.5.2000.en";
        let token = format!("{payload}.{}", signer("secret", payload));
        let c = verifier_lien("secret", &token, 1999).unwrap();
        assert_eq!((c.role, c.remove, c.log), (9, Some(4), Some(5)));
    }

    #[test]
    fn config_du_bot_et_lien() {
        let v = Verification {
            secret: Some("s".into()),
            bot_token: None,
            api: String::new(),
            http: reqwest::Client::new(),
            defis: Mutex::default(),
            par_lien: Mutex::default(),
            client_id: None,
            client_secret: None,
            serveurs: Mutex::default(),
            fichier_serveurs: std::env::temp_dir().join("misty-test-verify-guilds-2.json"),
        };
        let corps = br#"{"guild":"42","enabled":true,"role":"9","remove":"3","log":null,"lang":"fr"}"#;
        assert!(!v.recevoir_config(corps, "mauvaise"));
        assert!(v.lien_pour("42", 7).is_none());
        assert!(v.recevoir_config(corps, &signer("s", std::str::from_utf8(corps).unwrap())));
        let lien = v.lien_pour("42", 7).unwrap();
        let c = verifier_lien("s", &lien, maintenant()).unwrap();
        assert_eq!((c.guild, c.user, c.role, c.remove, c.log, c.lang.as_str()), (42, 7, 9, Some(3), None, "fr"));
        assert!(v.lien_pour("43", 7).is_none(), "serveur inconnu");
    }

    #[test]
    fn id_de_l_application() {
        // « 123456789012345678 » en base64, comme le début d'un token de bot
        assert_eq!(id_depuis_token("MTIzNDU2Nzg5MDEyMzQ1Njc4.abc.def").as_deref(), Some("123456789012345678"));
        assert_eq!(id_depuis_token("pas-un-token"), None);
    }

    #[test]
    fn essais() {
        let v = Verification {
            secret: Some("s".into()),
            bot_token: None,
            api: String::new(),
            http: reqwest::Client::new(),
            defis: Mutex::default(),
            par_lien: Mutex::default(),
            client_id: None,
            client_secret: None,
            serveurs: Mutex::default(),
            fichier_serveurs: std::env::temp_dir().join("misty-test-verify-guilds.json"),
        };
        let humain = [(0.0, 0.0), (40.0, 3.0), (80.0, 15.0), (120.0, 40.0), (180.0, 80.0), (240.0, 110.0), (300.0, 124.0), (380.0, 130.0)];
        let (id, y) = v.nouveau_defi("t").unwrap();
        let graine = v.defis.lock().unwrap()[&id].graine;
        let (x, py) = crate::puzzle::position(graine);
        assert_eq!(y, py);
        assert!(v.image(&id, false).unwrap().starts_with(b"\x89PNG"));
        assert!(v.image(&id, true).unwrap().starts_with(b"\x89PNG"));
        assert_eq!(v.repondre(&id, "autre lien", x as f32, &humain), Reponse::Perime);
        assert_eq!(v.repondre(&id, "t", x as f32 + 30.0, &humain), Reponse::Faux);
        assert_eq!(v.repondre(&id, "t", x as f32 + 2.0, &[(0.0, 0.0), (10.0, x as f32)]), Reponse::Faux, "geste de robot");
        assert_eq!(v.repondre(&id, "t", x as f32 + 30.0, &humain), Reponse::Perime, "3 essais ratés : défi jeté");
        assert_eq!(v.repondre(&id, "t", x as f32, &humain), Reponse::Perime);
        let (id, _) = v.nouveau_defi("t").unwrap();
        let x = crate::puzzle::position(v.defis.lock().unwrap()[&id].graine).0;
        assert_eq!(v.repondre(&id, "t", x as f32 - 3.0, &humain), Reponse::Juste);
        assert_eq!(v.repondre(&id, "t", x as f32, &humain), Reponse::Perime, "un défi ne sert qu'une fois");
        // au plus 10 images par lien
        for _ in 0..8 {
            assert!(v.nouveau_defi("t").is_some());
        }
        assert!(v.nouveau_defi("t").is_none());
    }
}
