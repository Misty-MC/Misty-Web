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

use hmac::{Hmac, Mac};
use rand::Rng;
use rand::seq::IndexedRandom;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Caractères sans ambiguïté (pas de 0/O, 1/I, 2/Z, 5/S, 8/B…).
const ALPHABET: &[u8] = b"ACDEFHJKLMNPRTUVWXY3479";
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
    let [v, g, u, r, log, exp, lang] = p.as_slice() else { return None };
    let c = Claims {
        guild: g.parse().ok()?,
        user: u.parse().ok()?,
        role: r.parse().ok()?,
        log: log.parse().ok().filter(|l| *l != 0),
        expires: exp.parse().ok()?,
        lang: lang.to_string(),
    };
    (*v == "v1" && c.expires > maintenant).then_some(c)
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
        Self { secret, bot_token, api, http, defis: Mutex::default(), par_lien: Mutex::default() }
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
    fn toutes_les_lettres_ont_un_glyphe() {
        for c in ALPHABET {
            assert!(glyphe(*c).iter().any(|l| l.contains('#')), "{}", *c as char);
        }
    }

    #[test]
    fn essais() {
        let v = Verification { secret: Some("s".into()), bot_token: None, api: String::new(), http: reqwest::Client::new(), defis: Mutex::default(), par_lien: Mutex::default() };
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
