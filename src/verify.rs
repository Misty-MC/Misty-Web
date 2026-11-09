//! Vérification Discord : captcha pour les nouveaux membres des serveurs qui utilisent Misty Bot.
//!
//! 1. Dans Discord, le membre clique sur « Vérifier » : le bot lui donne un lien signé
//!    `/verify?t=v1.<serveur>.<membre>.<rôle>.<salon de logs ou 0>.<expiration>.<langue>.<signature>`
//!    (HMAC-SHA256 avec `VERIFY_SECRET`, le même secret que le bot ; valable 10 minutes).
//! 2. Le site vérifie la signature, affiche un captcha en pixel art (un SVG généré à chaque
//!    fois : caractères dessinés pixel par pixel, mélangés à des pixels parasites).
//! 3. Réponse juste : le site donne le rôle lui-même via l'API Discord (`DISCORD_TOKEN`, le token
//!    du bot) et note la vérification dans le salon de logs. Le bot n'a pas besoin d'être joignable.
//!
//! Limites : 3 essais par image, 10 images par lien, et un champ piège invisible pour les robots.
//!
//! **Dans Discord (Activité)** : le bouton « Vérifier » ouvre le site *dans* Discord
//! (`https://<application>.discordsays.com`, relayé vers ce site). La page `/activity` utilise le
//! SDK Embedded App de Discord pour savoir qui est le membre (autorisation OAuth2 `identify`,
//! code échangé ici avec `DISCORD_CLIENT_SECRET`) et dans quel serveur il est. Le rôle et le salon
//! de logs de ce serveur viennent du bot, qui les envoie signés à `/api/verify/config`.

use hmac::{Hmac, Mac};
use rand::Rng;
use rand::seq::IndexedRandom;
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Caractères sans ambiguïté (pas de 0/O, 1/I, 2/Z, 5/S, 8/B, 6/E, U/V, N/M…).
const ALPHABET: &[u8] = b"ACDEFHJKLMPRTVWXY3479";
const LONGUEUR: usize = 5;
const ESSAIS_PAR_IMAGE: u32 = 3;
const IMAGES_PAR_LIEN: u32 = 10;
const VIE_IMAGE: Duration = Duration::from_secs(600);

/// Police 5×7 (une chaîne par ligne, `#` = pixel allumé).
fn glyphe(c: u8) -> [&'static str; 7] {
    match c {
        b'A' => [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        b'C' => [".####", "#....", "#....", "#....", "#....", "#....", ".####"],
        b'D' => ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."],
        b'E' => ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
        b'F' => ["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
        b'H' => ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
        b'J' => ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."],
        b'K' => ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
        b'L' => ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
        b'M' => ["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"],
        b'N' => ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"],
        b'P' => ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
        b'R' => ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
        b'T' => ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
        b'U' => ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
        b'V' => ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
        b'W' => ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"],
        b'X' => ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"],
        b'Y' => ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
        b'3' => ["####.", "....#", "....#", ".###.", "....#", "....#", "####."],
        b'4' => ["#..#.", "#..#.", "#..#.", "#####", "...#.", "...#.", "...#."],
        b'6' => [".###.", "#....", "#....", "####.", "#...#", "#...#", ".###."],
        b'7' => ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."],
        b'9' => [".###.", "#...#", "#...#", ".####", "....#", "....#", ".###."],
        _ => [".....", ".....", ".....", ".....", ".....", ".....", "....."],
    }
}

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
    code: String,
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

    /// Nouvelle image pour ce lien ; `None` après trop d'images.
    pub fn nouveau_defi(&self, token: &str) -> Option<String> {
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
        let code: String = (0..LONGUEUR).map(|_| *ALPHABET.choose(&mut rng).unwrap() as char).collect();
        let id: String = (0..24).map(|_| format!("{:x}", rng.random_range(0..16u8))).collect();
        let mut defis = self.defis.lock().unwrap();
        defis.retain(|_, d| d.cree.elapsed() < VIE_IMAGE);
        if defis.len() > 20_000 {
            return None;
        }
        defis.insert(id.clone(), Defi { code, token: token.to_string(), cree: Instant::now(), essais: 0 });
        Some(id)
    }

    /// L'image d'un défi (SVG), tant qu'il est valide.
    pub fn image(&self, id: &str) -> Option<String> {
        let defis = self.defis.lock().unwrap();
        let d = defis.get(id).filter(|d| d.cree.elapsed() < VIE_IMAGE)?;
        Some(dessiner(&d.code))
    }

    pub fn repondre(&self, id: &str, token: &str, essai: &str) -> Reponse {
        let mut defis = self.defis.lock().unwrap();
        let Some(d) = defis.get_mut(id).filter(|d| d.token == token && d.cree.elapsed() < VIE_IMAGE) else {
            return Reponse::Perime;
        };
        d.essais += 1;
        let propre: String = essai.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_ascii_uppercase();
        if propre == d.code {
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

/// Le captcha : chaque caractère dessiné pixel par pixel (police 5×7), décalé et incliné au
/// hasard, au milieu de pixels parasites de mêmes tailles et couleurs, et de deux traits.
pub fn dessiner(code: &str) -> String {
    let mut rng = rand::rng();
    let (w, h) = (330, 100);
    let couleurs = ["#c8dbff", "#e2ecff", "#9cc0ff", "#b6ccff"];
    let mut pixels: Vec<String> = Vec::new();
    let px = 7.0;
    for (i, c) in code.bytes().enumerate() {
        let x0 = 28.0 + i as f32 * 58.0 + rng.random_range(-4.0..4.0);
        let y0 = 24.0 + rng.random_range(-7.0..7.0);
        let angle: f32 = rng.random_range(-9.0..9.0);
        let (cx, cy) = (x0 + 2.5 * px, y0 + 3.5 * px);
        let (sin, cos) = angle.to_radians().sin_cos();
        let couleur = couleurs.choose(&mut rng).unwrap();
        for (ly, ligne) in glyphe(c).iter().enumerate() {
            for (lx, b) in ligne.bytes().enumerate() {
                if b != b'#' {
                    continue;
                }
                // rotation autour du centre du caractère, plus un léger tremblement
                let (dx, dy) = (x0 + lx as f32 * px - cx, y0 + ly as f32 * px - cy);
                let x = cx + dx * cos - dy * sin + rng.random_range(-0.6..0.6);
                let y = cy + dx * sin + dy * cos + rng.random_range(-0.6..0.6);
                pixels.push(format!(r#"<rect x="{x:.1}" y="{y:.1}" width="{px}" height="{px}" fill="{couleur}"/>"#));
            }
        }
    }
    // pixels parasites : plus petits et plus sombres que les caractères (lisibles pour un humain)
    let parasites = ["#6f9cff", "#4d6fb8", "#8fb1f5"];
    for _ in 0..55 {
        let taille = rng.random_range(3.0..5.5);
        let (x, y) = (rng.random_range(0.0..(w as f32 - taille)), rng.random_range(0.0..(h as f32 - taille)));
        let couleur = parasites.choose(&mut rng).unwrap();
        let opacite = rng.random_range(0.3..0.7);
        pixels.push(format!(r#"<rect x="{x:.1}" y="{y:.1}" width="{taille:.1}" height="{taille:.1}" fill="{couleur}" opacity="{opacite:.2}"/>"#));
    }
    // ordre mélangé : l'ordre du fichier ne trahit pas les caractères
    use rand::seq::SliceRandom;
    pixels.shuffle(&mut rng);
    let mut traits = String::new();
    for _ in 0..2 {
        let mut d = format!("M0 {:.0}", rng.random_range(15.0..80.0));
        for k in 1..=6 {
            d.push_str(&format!(" L{} {:.0}", k * w / 6, rng.random_range(10.0..86.0)));
        }
        traits.push_str(&format!(r##"<path d="{d}" stroke="#6f9cff" stroke-width="2" fill="none" opacity="0.35"/>"##));
    }
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" shape-rendering="crispEdges"><rect width="{w}" height="{h}" fill="#0f1830"/>{}{traits}</svg>"##,
        pixels.concat()
    )
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

    /// `PREVIEW_DIR=… cargo test apercu_captcha -- --ignored` : quelques images et leurs codes.
    #[test]
    #[ignore]
    fn apercu_captcha() {
        let dir = std::env::var("PREVIEW_DIR").unwrap();
        let mut codes = String::new();
        for i in 0..6 {
            let mut rng = rand::rng();
            let code: String = (0..LONGUEUR).map(|_| *ALPHABET.choose(&mut rng).unwrap() as char).collect();
            std::fs::write(format!("{dir}/c{i}.svg"), dessiner(&code)).unwrap();
            codes.push_str(&format!("{i} {code}\n"));
        }
        std::fs::write(format!("{dir}/codes.txt"), codes).unwrap();
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
    fn toutes_les_lettres_ont_un_glyphe() {
        for c in ALPHABET {
            assert!(glyphe(*c).iter().any(|l| l.contains('#')), "{}", *c as char);
        }
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
        let id = v.nouveau_defi("t").unwrap();
        let code = v.defis.lock().unwrap()[&id].code.clone();
        assert!(v.image(&id).unwrap().starts_with("<svg"));
        assert_eq!(v.repondre(&id, "autre lien", &code), Reponse::Perime);
        assert_eq!(v.repondre(&id, "t", "XXXXXX"), Reponse::Faux);
        assert_eq!(v.repondre(&id, "t", &code.to_lowercase()), Reponse::Juste);
        assert_eq!(v.repondre(&id, "t", &code), Reponse::Perime, "une image ne sert qu'une fois");
        // 3 essais faux : l'image est jetée
        let id = v.nouveau_defi("t").unwrap();
        for _ in 0..2 {
            assert_eq!(v.repondre(&id, "t", "nope"), Reponse::Faux);
        }
        assert_eq!(v.repondre(&id, "t", "nope"), Reponse::Perime);
        // au plus 10 images par lien
        for _ in 0..8 {
            assert!(v.nouveau_defi("t").is_some());
        }
        assert!(v.nouveau_defi("t").is_none());
    }
}
