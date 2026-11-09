//! Copie locale des fichiers de la dernière version (installeur et portable).
//!
//! `/download/…` sert d'abord ces copies : un téléchargement ne dépend ni du quota de l'API
//! GitHub, ni d'une lenteur de GitHub. Les fichiers sont téléchargés en arrière-plan dans un
//! fichier temporaire, vérifiés (taille annoncée) puis renommés : une copie présente est
//! toujours complète. Seule la dernière version est gardée.

use crate::github::{Kind, Release};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub struct Miroir {
    dossier: PathBuf,
    /// Client sans délai total (gros fichiers) mais avec délai de connexion et de lecture
    http: reqwest::Client,
}

impl Miroir {
    pub fn new(dossier: PathBuf) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("misty-site")
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(30))
            .build()
            .expect("client HTTP");
        Self { dossier, http }
    }

    /// Client pour relayer un fichier depuis GitHub (pas de délai total).
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// La copie locale d'un fichier, si elle est complète.
    pub fn fichier(&self, nom: &str, taille: u64) -> Option<PathBuf> {
        let chemin = self.dossier.join(nom_sur(nom)?);
        let meta = std::fs::metadata(&chemin).ok()?;
        (meta.is_file() && (taille == 0 || meta.len() == taille)).then_some(chemin)
    }

    /// Télécharge les fichiers de cette version qui manquent, puis supprime les anciens.
    pub async fn synchroniser(&self, release: &Release) {
        if tokio::fs::create_dir_all(&self.dossier).await.is_err() {
            return;
        }
        let mut gardes = Vec::new();
        for kind in [Kind::Setup, Kind::Portable] {
            let Some(asset) = release.asset(kind) else { continue };
            let Some(nom) = nom_sur(&asset.name) else { continue };
            gardes.push(nom.to_string());
            if self.fichier(&asset.name, asset.size).is_some() {
                continue;
            }
            match self.telecharger(&asset.browser_download_url, nom, asset.size).await {
                Ok(n) => eprintln!("Miroir : {nom} copié ({} Mo)", n / 1_048_576),
                Err(e) => eprintln!("Miroir : {nom} non copié ({e}), les téléchargements passent par GitHub"),
            }
        }
        // ménage : anciennes versions et fichiers temporaires abandonnés
        if gardes.is_empty() {
            return;
        }
        if let Ok(mut dir) = tokio::fs::read_dir(&self.dossier).await {
            while let Ok(Some(e)) = dir.next_entry().await {
                let nom = e.file_name().to_string_lossy().to_string();
                if !gardes.contains(&nom) {
                    let _ = tokio::fs::remove_file(e.path()).await;
                }
            }
        }
    }

    async fn telecharger(&self, url: &str, nom: &str, taille: u64) -> Result<u64, String> {
        let tmp = self.dossier.join(format!("{nom}.part"));
        let mut rep = self
            .http
            .get(url)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| e.to_string())?;
        let mut f = tokio::fs::File::create(&tmp).await.map_err(|e| e.to_string())?;
        let mut n = 0u64;
        let resultat = async {
            while let Some(morceau) = rep.chunk().await.map_err(|e| e.to_string())? {
                n += morceau.len() as u64;
                f.write_all(&morceau).await.map_err(|e| e.to_string())?;
            }
            f.flush().await.map_err(|e| e.to_string())?;
            if taille != 0 && n != taille {
                return Err(format!("taille {n} au lieu de {taille}"));
            }
            Ok(())
        }
        .await;
        drop(f);
        match resultat {
            Ok(()) => {
                tokio::fs::rename(&tmp, self.dossier.join(nom)).await.map_err(|e| e.to_string())?;
                Ok(n)
            }
            Err(e) => {
                let _ = tokio::fs::remove_file(&tmp).await;
                Err(e)
            }
        }
    }
}

/// Nom de fichier sans chemin (un nom venant de GitHub ne doit jamais sortir du dossier).
fn nom_sur(nom: &str) -> Option<&str> {
    let ok = !nom.is_empty()
        && !nom.starts_with('.')
        && !nom.contains(['/', '\\'])
        && Path::new(nom).file_name().is_some_and(|f| f == nom);
    ok.then_some(nom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noms_refuses() {
        assert_eq!(nom_sur("Misty-Setup-0.3.4.exe"), Some("Misty-Setup-0.3.4.exe"));
        assert_eq!(nom_sur("../etc/passwd"), None);
        assert_eq!(nom_sur("a/b.zip"), None);
        assert_eq!(nom_sur(".."), None);
        assert_eq!(nom_sur(""), None);
    }
}
