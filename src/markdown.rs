//! Rendu des notes de release (Markdown GitHub) en HTML sûr.
//! - le HTML brut des notes est affiché comme du texte, jamais interprété ;
//! - seuls les liens http(s), mailto et ancres sont conservés ;
//! - les titres sont décalés de deux niveaux pour s'intégrer à la page.

use pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};

fn safe_url(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("https://")
        || u.starts_with("http://")
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
}
