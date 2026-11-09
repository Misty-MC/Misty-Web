//! Messages dans le salon Discord de l'équipe (variable `DISCORD_WEBHOOK`, l'adresse d'un
//! webhook ; sans elle, rien n'est envoyé) :
//! - chaque rapport de logs, avec ses erreurs principales et le journal en fichier joint ;
//! - chaque matin (8 h, heure de Paris en hiver), le résumé de la veille.

use serde_json::{Value, json};
use std::time::Duration;
use time::OffsetDateTime;

pub struct Discord {
    webhook: Option<String>,
    http: reqwest::Client,
}

/// Couleurs des messages.
const ROUGE: u32 = 0xFF5A6E;
const BLEU: u32 = 0x2F8CFF;
const VERT: u32 = 0x13B24C;

impl Discord {
    pub fn new() -> Self {
        let webhook = std::env::var("DISCORD_WEBHOOK")
            .ok()
            .map(|w| w.trim().to_string())
            .filter(|w| {
                w.starts_with("https://discord.com/api/webhooks/")
                    || w.starts_with("https://discordapp.com/api/webhooks/")
            });
        if webhook.is_none() && std::env::var_os("DISCORD_WEBHOOK").is_some() {
            eprintln!("DISCORD_WEBHOOK ignoré : ce n'est pas une adresse de webhook Discord");
        }
        let http = reqwest::Client::builder()
            .user_agent("misty-site")
            .timeout(Duration::from_secs(20))
            .build()
            .expect("client HTTP");
        Self { webhook, http }
    }

    pub fn actif(&self) -> bool {
        self.webhook.is_some()
    }

    /// Envoie un message (et un fichier joint). Une erreur est seulement notée.
    async fn envoyer(&self, message: Value, fichier: Option<(String, Vec<u8>)>) {
        let Some(url) = &self.webhook else { return };
        let requete = match fichier {
            Some((nom, octets)) => {
                let partie = reqwest::multipart::Part::bytes(octets)
                    .file_name(nom)
                    .mime_str("text/plain")
                    .expect("type du fichier");
                let formulaire = reqwest::multipart::Form::new()
                    .text("payload_json", message.to_string())
                    .part("files[0]", partie);
                self.http.post(url).multipart(formulaire)
            }
            None => self.http.post(url).json(&message),
        };
        match requete.send().await {
            Ok(r) if r.status().is_success() => {}
            Ok(r) => eprintln!("Discord : réponse {}", r.status()),
            Err(e) => eprintln!("Discord : {e}"),
        }
    }

    /// Un rapport de logs vient d'arriver.
    pub async fn rapport(&self, r: &serde_json::Map<String, Value>) {
        if !self.actif() {
            return;
        }
        let champ = |cle: &str| r.get(cle).and_then(Value::as_str).unwrap_or("?").to_string();
        let code = champ("code");
        let log = r.get("log").and_then(Value::as_str).unwrap_or("");
        let plantage = champ("reason") == "crash";
        let erreurs = lignes_importantes(log);
        let mut description = String::new();
        if !erreurs.is_empty() {
            description = format!("```\n{}\n```", erreurs);
        }
        let ram = r
            .get("ram_mo")
            .and_then(Value::as_u64)
            .map(|m| format!("{:.0} Go", m as f64 / 1024.0))
            .unwrap_or_else(|| "?".into());
        let message = json!({
            "username": "Misty",
            "allowed_mentions": { "parse": [] },
            "embeds": [{
                "title": format!("{} {code}", if plantage { "💥 Plantage" } else { "📨 Logs envoyés" }),
                "description": description,
                "color": if plantage { ROUGE } else { BLEU },
                "fields": [
                    { "name": "Jeu", "value": champ("game"), "inline": false },
                    { "name": "Misty", "value": champ("version"), "inline": true },
                    { "name": "Windows", "value": champ("os"), "inline": true },
                    { "name": "RAM", "value": ram, "inline": true },
                    { "name": "Langue", "value": champ("langue"), "inline": true },
                    { "name": "Installation", "value": format!("`{}`", champ("id").chars().take(8).collect::<String>()), "inline": true },
                ],
                "footer": { "text": if plantage { "Envoyé automatiquement après le plantage" } else { "Envoyé par le joueur depuis la console" } },
                "timestamp": maintenant_rfc3339(),
            }],
        });
        self.envoyer(message, Some((format!("{code}.log"), log.as_bytes().to_vec())))
            .await;
    }

    /// Résumé de la veille, chaque jour vers 8 h (7 h UTC).
    pub async fn resume_en_boucle(self: std::sync::Arc<Self>, dossier: std::path::PathBuf) {
        if !self.actif() {
            return;
        }
        loop {
            tokio::time::sleep(jusqu_a_7h_utc()).await;
            let hier = OffsetDateTime::now_utc().date().previous_day();
            let Some(hier) = hier else { continue };
            let d = dossier.clone();
            let Ok(chiffres) =
                tokio::task::spawn_blocking(move || crate::telemetrie::chiffres_du_jour(&d, hier))
                    .await
            else {
                continue;
            };
            self.resume(&chiffres).await;
            // pas deux résumés le même jour
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    }

    async fn resume(&self, c: &Value) {
        let n = |cle: &str| c[cle].as_u64().unwrap_or(0);
        let parties = n("games");
        let plantages = n("crashes");
        let taux = if parties > 0 {
            format!("{:.1} %", plantages as f64 * 100.0 / parties as f64)
        } else {
            "–".into()
        };
        let liste = |cle: &str| -> String {
            let Some(o) = c[cle].as_object() else { return "–".into() };
            let mut v: Vec<(&String, u64)> = o.iter().map(|(k, v)| (k, v.as_u64().unwrap_or(0))).collect();
            v.sort_by(|a, b| b.1.cmp(&a.1));
            let texte: Vec<String> = v.iter().take(6).map(|(k, n)| format!("{k} : **{n}**")).collect();
            if texte.is_empty() { "–".into() } else { texte.join("\n") }
        };
        let message = json!({
            "username": "Misty",
            "allowed_mentions": { "parse": [] },
            "embeds": [{
                "title": format!("📊 Résumé du {}", c["day"].as_str().unwrap_or("?")),
                "color": if plantages > 0 { BLEU } else { VERT },
                "fields": [
                    { "name": "Installations actives", "value": n("installs").to_string(), "inline": true },
                    { "name": "Parties", "value": parties.to_string(), "inline": true },
                    { "name": "Plantages", "value": format!("{plantages} ({taux})"), "inline": true },
                    { "name": "Parties par loader", "value": liste("games_by_loader"), "inline": true },
                    { "name": "Plantages par loader", "value": liste("crashes_by_loader"), "inline": true },
                    { "name": "Versions de Misty", "value": liste("launcher_versions"), "inline": true },
                    { "name": "Rapports de logs", "value": n("reports").to_string(), "inline": true },
                    { "name": "Mises à jour", "value": n("updates").to_string(), "inline": true },
                ],
                "timestamp": maintenant_rfc3339(),
            }],
        });
        self.envoyer(message, None).await;
    }
}

fn maintenant_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_default()
}

/// Temps jusqu'au prochain 7 h UTC.
fn jusqu_a_7h_utc() -> Duration {
    let maintenant = OffsetDateTime::now_utc();
    let mut cible = maintenant.replace_time(time::Time::from_hms(7, 0, 0).expect("heure"));
    if cible <= maintenant {
        cible += time::Duration::days(1);
    }
    Duration::from_secs((cible - maintenant).whole_seconds().max(60) as u64)
}

/// Les lignes qui expliquent le problème : erreurs, exceptions et le début de leurs traces
/// (au plus ~3 500 caractères, la limite d'un message étant 4 096).
fn lignes_importantes(log: &str) -> String {
    let mut gardees: Vec<&str> = Vec::new();
    let mut suite = 0;
    for ligne in log.lines() {
        let haut = ligne.to_ascii_uppercase();
        let erreur = haut.contains("/ERROR]")
            || haut.contains("/FATAL]")
            || haut.contains(" ERROR ")
            || ligne.contains("Exception")
            || ligne.contains("panicked at")
            || ligne.contains("[Misty]") && (haut.contains("CRASH") || haut.contains("PLANT"));
        if erreur {
            gardees.push(ligne);
            suite = 3;
        } else if suite > 0 && (ligne.trim_start().starts_with("at ") || ligne.trim_start().starts_with("Caused by")) {
            gardees.push(ligne);
            suite -= 1;
        } else {
            suite = 0;
        }
    }
    // les dernières erreurs sont les plus proches du plantage
    let mut texte = String::new();
    for l in gardees.iter().rev() {
        let l: String = l.replace("```", "'''").chars().take(300).collect();
        if texte.len() + l.len() + 1 > 3500 {
            break;
        }
        texte.insert_str(0, &format!("{l}\n"));
    }
    texte.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn erreurs_du_journal() {
        let log = "[10:00:00] [main/INFO]: ok\n[10:00:01] [Render thread/ERROR]: Boom\njava.lang.NullPointerException: x\n    at a.b(C.java:1)\n    at a.c(C.java:2)\n    at a.d(C.java:3)\n    at a.e(C.java:4)\n[10:00:02] [main/INFO]: fin";
        let t = super::lignes_importantes(log);
        assert!(t.starts_with("[10:00:01] [Render thread/ERROR]: Boom"));
        assert!(t.contains("NullPointerException"));
        assert!(t.contains("a.d(C.java:3)"));
        assert!(!t.contains("fin"));
    }

    #[test]
    fn prochain_resume() {
        let d = super::jusqu_a_7h_utc();
        assert!(d.as_secs() >= 60 && d.as_secs() <= 24 * 3600);
    }
}
