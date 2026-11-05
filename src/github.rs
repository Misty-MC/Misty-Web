//! Accès aux releases GitHub du launcher, avec un cache en mémoire de quelques minutes.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const CACHE_TTL: Duration = Duration::from_secs(120);
const USER_AGENT: &str = "misty-site";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: Option<String>,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    pub content_type: Option<String>,
}

/// Les deux fichiers publiés à chaque version.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// `Misty-Setup-<version>.exe`
    Setup,
    /// `Misty-<version>.zip`
    Portable,
}

impl Kind {
    pub fn from_slug(s: &str) -> Option<Kind> {
        match s {
            "setup" => Some(Kind::Setup),
            "portable" => Some(Kind::Portable),
            _ => None,
        }
    }

    pub fn matches(self, file: &str) -> bool {
        let f = file.to_ascii_lowercase();
        match self {
            Kind::Setup => f.starts_with("misty-setup-") && f.ends_with(".exe"),
            Kind::Portable => {
                f.starts_with("misty-") && !f.starts_with("misty-setup") && f.ends_with(".zip")
            }
        }
    }
}

impl Release {
    pub fn asset(&self, kind: Kind) -> Option<&Asset> {
        self.assets.iter().find(|a| kind.matches(&a.name))
    }

    pub fn title(&self) -> &str {
        self.name
            .as_deref()
            .filter(|n| !n.trim().is_empty())
            .unwrap_or(&self.tag_name)
    }

    pub fn notes(&self) -> &str {
        self.body.as_deref().unwrap_or("").trim()
    }
}

pub struct GitHub {
    http: reqwest::Client,
    api_base: String,
    repo: String,
    token: Option<String>,
    cache: Mutex<Option<(Instant, Arc<Vec<Release>>)>>,
}

impl GitHub {
    pub fn new(api_base: String, repo: String, token: Option<String>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("client HTTP");
        Self {
            http,
            api_base,
            repo,
            token,
            cache: Mutex::new(None),
        }
    }

    pub fn repo(&self) -> &str {
        &self.repo
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Toutes les releases publiées (sans les brouillons), les plus récentes d'abord.
    /// En cas d'erreur, renvoie la dernière liste connue si elle existe.
    pub async fn releases(&self) -> Option<Arc<Vec<Release>>> {
        let mut cache = self.cache.lock().await;
        if let Some((at, list)) = cache.as_ref()
            && at.elapsed() < CACHE_TTL
        {
            return Some(list.clone());
        }
        match self.fetch().await {
            Ok(list) => {
                let list = Arc::new(list);
                *cache = Some((Instant::now(), list.clone()));
                Some(list)
            }
            Err(e) => {
                eprintln!("GitHub indisponible : {e}");
                // On réessaiera au prochain appel ; en attendant, on sert l'ancienne liste.
                cache.as_ref().map(|(_, list)| list.clone())
            }
        }
    }

    async fn fetch(&self) -> Result<Vec<Release>, reqwest::Error> {
        let url = format!(
            "{}/repos/{}/releases?per_page=100",
            self.api_base, self.repo
        );
        let mut req = self
            .http
            .get(url)
            .header("Accept", "application/vnd.github+json");
        if let Some(t) = &self.token {
            req = req.bearer_auth(t);
        }
        let mut list: Vec<Release> = req.send().await?.error_for_status()?.json().await?;
        list.retain(|r| !r.draft);
        Ok(list)
    }
}

/// La version mise en avant : la plus récente stable, sinon la plus récente tout court.
pub fn latest(list: &[Release]) -> Option<&Release> {
    list.iter().find(|r| !r.prerelease).or_else(|| list.first())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_names() {
        assert!(Kind::Setup.matches("Misty-Setup-0.3.1.exe"));
        assert!(!Kind::Setup.matches("Misty-0.3.1.zip"));
        assert!(Kind::Portable.matches("Misty-0.3.1.zip"));
        assert!(!Kind::Portable.matches("Misty-Setup-0.3.1.zip"));
    }
}
