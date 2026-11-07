//! Rendu des notes de release (Markdown GitHub) en HTML sûr.
//! - le HTML brut des notes est affiché comme du texte, jamais interprété ;
//! - seuls les liens http(s), mailto et ancres sont conservés ;
//! - les titres sont décalés de deux niveaux pour s'intégrer à la page.
//!
//! `render_doc` sert aux pages de documentation : mêmes règles de sécurité, mais les titres
//! gardent leur niveau, reçoivent un identifiant (pour les ancres) et alimentent le sommaire.

use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use std::collections::HashSet;

fn safe_url(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("https://")
        || u.starts_with("http://")
        || (u.starts_with('/') && !u.starts_with("//"))
        || u.starts_with("mailto:")
        || u.starts_with('#')
}

fn shift(level: HeadingLevel) -> HeadingLevel {
    match level {
        HeadingLevel::H1 => HeadingLevel::H3,
        HeadingLevel::H2 => HeadingLevel::H4,
        HeadingLevel::H3 => HeadingLevel::H5,
        _ => HeadingLevel::H6,
    }
}

pub fn render(src: &str) -> String {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let events = Parser::new_ext(src, opts).map(|ev| match ev {
        Event::Html(s) | Event::InlineHtml(s) => Event::Text(s),
        Event::Start(Tag::Heading {
            level,
            id,
            classes,
            attrs,
        }) => Event::Start(Tag::Heading {
            level: shift(level),
            id,
            classes,
            attrs,
        }),
        Event::End(TagEnd::Heading(level)) => Event::End(TagEnd::Heading(shift(level))),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) if !safe_url(&dest_url) => Event::Start(Tag::Link {
            link_type,
            dest_url: CowStr::Borrowed("#"),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) if !safe_url(&dest_url) => Event::Start(Tag::Image {
            link_type,
            dest_url: CowStr::Borrowed(""),
            title,
            id,
        }),
        other => other,
    });
    let mut out = String::with_capacity(src.len() * 3 / 2);
    html::push_html(&mut out, events);
    out
}

/// Les premiers points d'une liste à puces, en texte brut (pour l'aperçu de la page d'accueil).
pub fn highlights(src: &str, max: usize) -> Vec<String> {
    src.lines()
        .filter_map(|l| {
            let l = l.trim_start();
            l.strip_prefix("- ")
                .or_else(|| l.strip_prefix("* "))
                .or_else(|| l.strip_prefix("+ "))
        })
        .map(|l| l.replace("**", "").replace('`', "").trim().to_string())
        .filter(|l| !l.is_empty())
        .take(max)
        .collect()
}

/// Une entrée du sommaire « Sur cette page ».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    pub level: u8,
    pub id: String,
    pub text: String,
}

/// Identifiant d'ancre : minuscules, lettres et chiffres (accents compris), le reste en tirets.
pub fn slugify(text: &str) -> String {
    let mut out = String::new();
    for c in text.trim().to_lowercase().chars() {
        if c.is_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn level_num(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Rend une page de documentation et renvoie son sommaire (titres de niveau 2 et 3).
/// Les encadrés GitHub (`> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`…) sont pris en charge.
pub fn render_doc(src: &str) -> (String, Vec<TocEntry>) {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_GFM;
    let mut out: Vec<Event> = Vec::new();
    let mut toc = Vec::new();
    let mut used = HashSet::new();
    // Titre en cours : niveau, événements intérieurs, texte brut.
    let mut heading: Option<(u8, Vec<Event>, String)> = None;

    for ev in Parser::new_ext(src, opts) {
        let ev = match ev {
            Event::Html(s) | Event::InlineHtml(s) => Event::Text(s),
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) if !safe_url(&dest_url) => Event::Start(Tag::Link {
                link_type,
                dest_url: CowStr::Borrowed("#"),
                title,
                id,
            }),
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) if !safe_url(&dest_url) => Event::Start(Tag::Image {
                link_type,
                dest_url: CowStr::Borrowed(""),
                title,
                id,
            }),
            other => other,
        };
        match ev {
            Event::Start(Tag::Heading { level, .. }) => {
                heading = Some((level_num(level), Vec::new(), String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                let Some((lvl, inner, text)) = heading.take() else {
                    continue;
                };
                let base = match slugify(&text) {
                    s if s.is_empty() => "section".to_string(),
                    s => s,
                };
                let mut id = base.clone();
                let mut n = 2;
                while !used.insert(id.clone()) {
                    id = format!("{base}-{n}");
                    n += 1;
                }
                if lvl == 2 || lvl == 3 {
                    toc.push(TocEntry {
                        level: lvl,
                        id: id.clone(),
                        text: text.trim().to_string(),
                    });
                }
                out.push(Event::Html(format!("<h{lvl} id=\"{id}\">").into()));
                out.extend(inner);
                out.push(Event::Html(
                    format!(
                        "<a class=\"anchor\" href=\"#{id}\" aria-hidden=\"true\">#</a></h{lvl}>\n"
                    )
                    .into(),
                ));
            }
            ev => match heading.as_mut() {
                Some((_, inner, text)) => {
                    if let Event::Text(t) | Event::Code(t) = &ev {
                        text.push_str(t);
                    }
                    inner.push(ev);
                }
                None => out.push(ev),
            },
        }
    }
    let mut html_out = String::with_capacity(src.len() * 3 / 2);
    html::push_html(&mut html_out, out.into_iter());
    (html_out, toc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_html_is_escaped() {
        let out = render("Hello <script>alert(1)</script>");
        assert!(!out.contains("<script>"));
        assert!(out.contains("&lt;script&gt;"));
    }

    #[test]
    fn unsafe_links_are_neutralised() {
        let out = render("[x](javascript:alert(1)) [y](https://example.com)");
        assert!(!out.contains("javascript:"));
        assert!(out.contains("https://example.com"));
    }

    #[test]
    fn headings_are_shifted() {
        assert!(render("# Titre").contains("<h3>"));
        assert!(render("## Titre").contains("<h4>"));
    }

    #[test]
    fn highlights_take_bullets() {
        let h = highlights(
            "## New\n- **Faster** start\n- `.mrpack` import\n\ntext\n* third\n- fourth",
            3,
        );
        assert_eq!(h, vec!["Faster start", ".mrpack import", "third"]);
    }

    #[test]
    fn doc_headings_get_ids_and_toc() {
        let (html, toc) = render_doc(
            "# Titre\n\n## Premier lancement\n\ntexte\n\n### `data.txt`\n\n## Premier lancement",
        );
        assert!(html.contains(r#"<h2 id="premier-lancement">"#));
        assert!(html.contains(r#"<h2 id="premier-lancement-2">"#));
        assert!(html.contains(r#"<h3 id="data-txt">"#));
        assert_eq!(toc.len(), 3);
        assert_eq!(
            toc[1],
            TocEntry {
                level: 3,
                id: "data-txt".into(),
                text: "data.txt".into()
            }
        );
    }

    #[test]
    fn doc_is_still_safe_and_keeps_internal_links() {
        let (html, _) = render_doc(
            "<script>x</script>\n\n[a](/docs/profiles) [b](//evil.com) [c](javascript:x)",
        );
        assert!(!html.contains("<script>"));
        assert!(html.contains(r#"href="/docs/profiles""#));
        assert!(!html.contains("evil.com") && !html.contains("javascript:"));
    }

    #[test]
    fn doc_alerts() {
        let (html, _) = render_doc("> [!WARNING]\n> Attention");
        assert!(html.contains("markdown-alert-warning"));
    }
}
