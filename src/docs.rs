//! Documentation : pages Markdown rangées dans `docs/<langue>/`, intégrées au binaire.
//!
//! Chaque fichier s'appelle `NN-slug.md` (NN = ordre dans sa section) et commence par
//! un en-tête TOML entre `+++` :
//!
//! ```text
//! +++
//! title = "Installation"
//! description = "Installeur ou version portable…"
//! section = "start"
//! +++
//! ```
//!
//! L'anglais sert de référence : une langue sans dossier affiche la doc anglaise.

use crate::i18n::{Lang, Tr};
use crate::markdown::{self, TocEntry};
use include_dir::{Dir, include_dir};
use serde::Deserialize;
use std::sync::LazyLock;

static DOCS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/docs");

/// Langue de référence, complète par définition.
const REFERENCE: &str = "en";

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    Start,
    Play,
    Tools,
    Help,
    Dev,
}

impl Section {
    pub const ALL: [Section; 5] = [
        Section::Start,
        Section::Play,
        Section::Tools,
        Section::Help,
        Section::Dev,
    ];

    pub fn label(self, tr: &Tr) -> &str {
        match self {
            Section::Start => &tr.docs_sec_start,
            Section::Play => &tr.docs_sec_play,
            Section::Tools => &tr.docs_sec_tools,
            Section::Help => &tr.docs_sec_help,
            Section::Dev => &tr.docs_sec_dev,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrontMatter {
    title: String,
    description: String,
    section: Section,
}

pub struct DocPage {
    pub slug: String,
    /// Chemin dans le dépôt, pour le lien « Modifier sur GitHub ».
    pub path: String,
    pub title: String,
    pub description: String,
    pub section: Section,
    pub html: String,
    pub toc: Vec<TocEntry>,
    order: u32,
}

/// Toutes les pages d'une langue, dans l'ordre de lecture.
pub struct DocSet {
    pub lang_dir: &'static str,
    pub pages: Vec<DocPage>,
}

impl DocSet {
    pub fn find(&self, slug: &str) -> Option<usize> {
        self.pages.iter().position(|p| p.slug == slug)
    }
    pub fn in_section(&self, s: Section) -> impl Iterator<Item = &DocPage> {
        self.pages.iter().filter(move |p| p.section == s)
    }
}

fn parse_file(lang_dir: &str, name: &str, src: &str) -> DocPage {
    let ctx = format!("docs/{lang_dir}/{name}");
    let rest = src
        .strip_prefix("+++\n")
        .unwrap_or_else(|| panic!("{ctx} : l'en-tête +++ manque"));
    let (front, body) = rest
        .split_once("\n+++\n")
        .unwrap_or_else(|| panic!("{ctx} : en-tête +++ non refermé"));
    let fm: FrontMatter =
        toml::from_str(front).unwrap_or_else(|e| panic!("{ctx} : en-tête invalide : {e}"));
    let stem = name
        .strip_suffix(".md")
        .unwrap_or_else(|| panic!("{ctx} : extension .md attendue"));
    let (num, slug) = stem
        .split_once('-')
        .unwrap_or_else(|| panic!("{ctx} : nom attendu « NN-slug.md »"));
    let order = num
        .parse()
        .unwrap_or_else(|_| panic!("{ctx} : « {num} » n'est pas un numéro"));
    let (html, toc) = markdown::render_doc(body);
    DocPage {
        slug: slug.to_string(),
        path: format!("docs/{lang_dir}/{name}"),
        title: fm.title,
        description: fm.description,
        section: fm.section,
        html,
        toc,
        order,
    }
}

fn load(lang_dir: &'static str) -> Option<DocSet> {
    let dir = DOCS_DIR.get_dir(lang_dir)?;
    let mut pages: Vec<DocPage> = dir
        .files()
        .filter_map(|f| {
            let name = f.path().file_name()?.to_str()?;
            name.ends_with(".md").then(|| {
                let src = f
                    .contents_utf8()
                    .unwrap_or_else(|| panic!("{name} : UTF-8 attendu"));
                parse_file(lang_dir, name, &src.replace("\r\n", "\n"))
            })
        })
        .collect();
    let rank = |s: Section| Section::ALL.iter().position(|x| *x == s).unwrap();
    pages.sort_by_key(|p| (rank(p.section), p.order));
    Some(DocSet { lang_dir, pages })
}

static DOCS: LazyLock<Vec<DocSet>> =
    LazyLock::new(|| Lang::ALL.iter().filter_map(|l| load(l.code())).collect());

/// Charge la documentation (appelé au démarrage pour échouer tôt en cas d'erreur).
pub fn init() {
    assert!(
        get(Lang::En).lang_dir == REFERENCE,
        "docs/{REFERENCE} est obligatoire"
    );
}

/// La documentation dans cette langue, ou en anglais si elle n'est pas traduite.
pub fn get(lang: Lang) -> &'static DocSet {
    DOCS.iter()
        .find(|d| d.lang_dir == lang.code())
        .or_else(|| DOCS.iter().find(|d| d.lang_dir == REFERENCE))
        .expect("docs/en manquant")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les liens encodent les accents (`%C3%A9`) ; les navigateurs les décodent avant de chercher l'ancre.
    fn percent_decode(s: &str) -> String {
        let b = s.as_bytes();
        let mut out = Vec::with_capacity(b.len());
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'%'
                && i + 2 < b.len()
                && let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16)
            {
                out.push(v);
                i += 3;
                continue;
            }
            out.push(b[i]);
            i += 1;
        }
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn every_translation_matches_english() {
        let en = get(Lang::En);
        assert!(!en.pages.is_empty());
        for set in DOCS.iter() {
            let slugs: Vec<_> = set.pages.iter().map(|p| (&p.slug, p.section)).collect();
            let ref_slugs: Vec<_> = en.pages.iter().map(|p| (&p.slug, p.section)).collect();
            assert_eq!(
                slugs, ref_slugs,
                "docs/{} n'a pas les mêmes pages que docs/en",
                set.lang_dir
            );
        }
    }

    #[test]
    fn internal_links_point_to_existing_pages() {
        for set in DOCS.iter() {
            for p in &set.pages {
                for part in p.html.split("href=\"/docs/").skip(1) {
                    let link = part.split('"').next().unwrap();
                    let (target, anchor) = link.split_once('#').unwrap_or((link, ""));
                    if target.is_empty() {
                        continue;
                    }
                    let idx = set.find(target);
                    assert!(
                        idx.is_some(),
                        "{} : lien vers /docs/{target} introuvable",
                        p.path
                    );
                    if !anchor.is_empty() {
                        let anchor = percent_decode(anchor);
                        let html = &set.pages[idx.unwrap()].html;
                        assert!(
                            html.contains(&format!("id=\"{anchor}\"")),
                            "{} : ancre #{anchor} introuvable dans /docs/{target}",
                            p.path
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn slugs_are_unique_and_titles_set() {
        for set in DOCS.iter() {
            let mut seen = std::collections::HashSet::new();
            for p in &set.pages {
                assert!(seen.insert(&p.slug), "{} : slug en double", p.path);
                assert!(
                    !p.title.is_empty() && !p.description.is_empty(),
                    "{} : titre ou description vide",
                    p.path
                );
            }
        }
    }

    #[test]
    fn untranslated_languages_fall_back_to_english() {
        assert_eq!(
            get(Lang::Ru).lang_dir,
            if DOCS.iter().any(|d| d.lang_dir == "ru") {
                "ru"
            } else {
                "en"
            }
        );
    }
}
