//! Icônes SVG en ligne (traits 24×24, couleur héritée du texte).

use maud::{Markup, PreEscaped};

fn icon(paths: &str) -> Markup {
    PreEscaped(format!(
        r#"<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{paths}</svg>"#
    ))
}

pub fn download() -> Markup {
    icon(r#"<path d="M12 3v12m0 0l-5-5m5 5l5-5M4 20h16"/>"#)
}
pub fn archive() -> Markup {
    icon(r#"<path d="M21 8v13H3V8M1 3h22v5H1zM10 12h4"/>"#)
}
pub fn notes() -> Markup {
    icon(
        r#"<path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><path d="M14 3v6h6M8 13h8M8 17h5"/>"#,
    )
}
pub fn external() -> Markup {
    icon(r#"<path d="M7 17L17 7M8 7h9v9"/>"#)
}
pub fn arrow_up() -> Markup {
    icon(r#"<path d="M12 19V5M5 12l7-7 7 7"/>"#)
}
pub fn menu() -> Markup {
    icon(r#"<path d="M4 6h16M4 12h16M4 18h16"/>"#)
}
pub fn search() -> Markup {
    icon(r#"<circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/>"#)
}
pub fn chevron() -> Markup {
    icon(r#"<path d="M6 9l6 6 6-6"/>"#)
}
pub fn check() -> Markup {
    icon(r#"<path d="M5 12l5 5 9-10"/>"#)
}
pub fn globe() -> Markup {
    icon(
        r#"<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/>"#,
    )
}
pub fn bolt() -> Markup {
    icon(r#"<path d="M13 2L4 14h7l-1 8 9-12h-7z"/>"#)
}
pub fn grid() -> Markup {
    icon(
        r#"<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>"#,
    )
}
pub fn user() -> Markup {
    icon(r#"<circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/>"#)
}
pub fn refresh() -> Markup {
    icon(r#"<path d="M21 12a9 9 0 1 1-3-6.7L21 8"/><path d="M21 3v5h-5"/>"#)
}
pub fn cube() -> Markup {
    icon(r#"<path d="M12 2l9 5v10l-9 5-9-5V7z"/><path d="M3 7l9 5 9-5M12 12v10"/>"#)
}
pub fn layers() -> Markup {
    icon(r#"<path d="M12 2l10 5-10 5L2 7z"/><path d="M2 12l10 5 10-5M2 17l10 5 10-5"/>"#)
}
pub fn brush() -> Markup {
    icon(r#"<path d="M18 2l4 4-11 11-4-4z"/><path d="M7 13l-3 3a3 3 0 0 0 4 4l3-3"/>"#)
}
pub fn package() -> Markup {
    icon(
        r#"<path d="M16.5 9.4L7.5 4.2M21 16V8l-9-5-9 5v8l9 5z"/><path d="M3.3 7L12 12l8.7-5M12 22V12"/>"#,
    )
}
pub fn windows() -> Markup {
    PreEscaped(
        r#"<svg class="ic" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M3 5.5l7.5-1v7H3zM11.5 4.4L21 3v8.5h-9.5zM3 12.5h7.5v7L3 18.5zM11.5 12.5H21V21l-9.5-1.4z"/></svg>"#.into(),
    )
}
pub fn github() -> Markup {
    PreEscaped(
        r#"<svg class="ic" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 .5a11.5 11.5 0 0 0-3.6 22.4c.6.1.8-.3.8-.6v-2c-3.2.7-3.9-1.5-3.9-1.5-.5-1.3-1.3-1.7-1.3-1.7-1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.8 1.3 3.5 1 .1-.8.4-1.3.7-1.6-2.6-.3-5.3-1.3-5.3-5.7 0-1.3.5-2.3 1.2-3.1-.1-.3-.5-1.5.1-3.1 0 0 1-.3 3.2 1.2a11 11 0 0 1 5.8 0c2.2-1.5 3.2-1.2 3.2-1.2.6 1.6.2 2.8.1 3.1.8.8 1.2 1.8 1.2 3.1 0 4.4-2.7 5.4-5.3 5.7.4.4.8 1.1.8 2.2v3.3c0 .3.2.7.8.6A11.5 11.5 0 0 0 12 .5z"/></svg>"#.into(),
    )
}
pub fn rust() -> Markup {
    icon(
        r#"<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3"/><path d="M12 1v3M12 20v3M1 12h3M20 12h3M4.2 4.2l2.1 2.1M17.7 17.7l2.1 2.1M4.2 19.8l2.1-2.1M17.7 6.3l2.1-2.1"/>"#,
    )
}
