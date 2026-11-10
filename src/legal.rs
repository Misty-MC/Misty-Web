//! Pages légales : mentions légales (`/legal`), confidentialité (`/privacy`) et conditions
//! d'utilisation (`/terms`), en Markdown dans `legal/<fr|en>/`, intégrées au binaire.
//!
//! Chaque fichier commence par un en-tête TOML entre `+++` (`title`, `description`, `updated`
//! au format AAAA-MM-JJ). Elles existent en français et en anglais ; les autres langues
//! affichent la version anglaise.

use crate::i18n::{Lang, Tr};
use crate::markdown::{self, TocEntry};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Notice,
    Privacy,
    Terms,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Notice, Kind::Privacy, Kind::Terms];

    pub fn path(self) -> &'static str {
        match self {
            Kind::Notice => "/legal",
            Kind::Privacy => "/privacy",
            Kind::Terms => "/terms",
        }
    }

    /// Nom court, pour le pied de page et le menu des pages légales.
    pub fn label(self, tr: &Tr) -> &str {
        match self {
            Kind::Notice => &tr.legal_notice,
            Kind::Privacy => &tr.legal_privacy,
            Kind::Terms => &tr.legal_terms,
        }
    }

    fn file(self) -> &'static str {
        match self {
            Kind::Notice => "notice",
            Kind::Privacy => "privacy",
            Kind::Terms => "terms",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrontMatter {
    title: String,
    description: String,
    updated: String,
}

pub struct LegalPage {
    pub kind: Kind,
    /// `fr` ou `en` : la langue dans laquelle la page est écrite.
    pub lang_dir: &'static str,
    /// Chemin dans le dépôt, pour le lien « Modifier sur GitHub ».
    pub path: String,
    pub title: String,
    pub description: String,
    pub updated: time::Date,
    pub html: String,
    pub toc: Vec<TocEntry>,
}

fn source(lang_dir: &str, kind: Kind) -> &'static str {
    match (lang_dir, kind) {
        ("fr", Kind::Notice) => include_str!("../legal/fr/notice.md"),
        ("fr", Kind::Privacy) => include_str!("../legal/fr/privacy.md"),
        ("fr", Kind::Terms) => include_str!("../legal/fr/terms.md"),
        (_, Kind::Notice) => include_str!("../legal/en/notice.md"),
        (_, Kind::Privacy) => include_str!("../legal/en/privacy.md"),
        (_, Kind::Terms) => include_str!("../legal/en/terms.md"),
    }
}

fn parse(lang_dir: &'static str, kind: Kind) -> LegalPage {
    let path = format!("legal/{lang_dir}/{}.md", kind.file());
    let src = source(lang_dir, kind).replace("\r\n", "\n");
    let rest = src
        .strip_prefix("+++\n")
        .unwrap_or_else(|| panic!("{path} : l'en-tête +++ manque"));
    let (front, body) = rest
        .split_once("\n+++\n")
        .unwrap_or_else(|| panic!("{path} : en-tête +++ non refermé"));
    let fm: FrontMatter =
        toml::from_str(front).unwrap_or_else(|e| panic!("{path} : en-tête invalide : {e}"));
    let updated = time::Date::parse(&fm.updated, time::macros::format_description!("[year]-[month]-[day]"))
        .unwrap_or_else(|e| panic!("{path} : date `updated` invalide : {e}"));
    let (html, toc) = markdown::render_doc(body);
    LegalPage { kind, lang_dir, path, title: fm.title, description: fm.description, updated, html, toc }
}

/// Les pages, par langue écrite (`fr` puis `en`) et dans l'ordre de `Kind::ALL`.
static PAGES: LazyLock<Vec<LegalPage>> = LazyLock::new(|| {
    ["fr", "en"]
        .into_iter()
        .flat_map(|l| Kind::ALL.into_iter().map(move |k| parse(l, k)))
        .collect()
});

/// Charge les pages (appelé au démarrage pour échouer tôt en cas d'erreur).
pub fn init() {
    LazyLock::force(&PAGES);
}

/// La page dans cette langue : le français pour le français, l'anglais pour les autres.
pub fn get(lang: Lang, kind: Kind) -> &'static LegalPage {
    let dir = if lang == Lang::Fr { "fr" } else { "en" };
    PAGES
        .iter()
        .find(|p| p.lang_dir == dir && p.kind == kind)
        .expect("toutes les pages légales sont chargées")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_loads_in_french_and_english() {
        for kind in Kind::ALL {
            let fr = get(Lang::Fr, kind);
            let en = get(Lang::En, kind);
            assert_eq!((fr.lang_dir, en.lang_dir), ("fr", "en"));
            assert!(!fr.html.is_empty() && !en.html.is_empty());
            assert_ne!(fr.title, en.title);
            // mêmes sections dans les deux langues
            assert_eq!(fr.toc.len(), en.toc.len(), "{kind:?} : sommaires différents");
        }
    }

    #[test]
    fn other_languages_get_english() {
        for lang in [Lang::Es, Lang::It, Lang::Ru] {
            assert_eq!(get(lang, Kind::Privacy).lang_dir, "en");
        }
    }
}
