//! Page d'accueil.

use super::icons;
use super::layout::{self, Head, Page};
use crate::format;
use crate::github::{Kind, Release};
use crate::markdown;
use maud::{Markup, PreEscaped, html};

/// Mod loaders proposés par le launcher (Quilt y est désactivé).
const LOADERS: [&str; 5] = ["Forge", "NeoForge", "Fabric", "OptiFine", "LiteLoader"];

pub fn render(p: &Page, latest: Option<&Release>, scene: &str) -> Markup {
    let tr = p.tr;
    let main = html! {
        (hero(p, latest, scene))
        (preview(p))
        (layout::fog_divider(p))

        section.sec #features {
            div.wrap {
                div.head.reveal {
                    span.label { (tr.feat_label) }
                    h2 { (tr.feat_title) }
                    p { (tr.feat_text) }
                }
                div.cards {
                    (card(icons::bolt(), &tr.c1_t, &tr.c1_p))
                    (card(icons::grid(), &tr.c2_t, &tr.c2_p))
                    (card(icons::user(), &tr.c3_t, &tr.c3_p))
                    (card(icons::refresh(), &tr.c4_t, &tr.c4_p))
                    (card(icons::globe(), &tr.c5_t, &tr.c5_p))
                    (card(icons::layers(), &tr.c6_t, &tr.c6_p))
                }
            }
        }

        section.sec #loaders {
            div.wrap.loaders {
                div.reveal {
                    span.label { (tr.load_label) }
                    div.big { (LOADERS.len()) }
                    p.big-cap { (tr.load_caption) }
                    h2 { (tr.load_title) }
                    p.lead { (tr.load_text) }
                    div.sources {
                        span.small { (tr.load_sources) }
                        span.pill { i.sw style="--c:#1bd96a" {} "Modrinth" }
                        span.pill { i.sw style="--c:#f16436" {} "CurseForge" }
                    }
                }
                ol.rows.reveal {
                    @for (i, name) in LOADERS.iter().enumerate() {
                        li {
                            span.n { (format!("{:02}", i + 1)) }
                            span.name { (name) }
                            span.ok { (icons::check()) (tr.load_item) }
                        }
                    }
                }
            }
        }

        (download(p, latest))
        (faq(p))

        section.cta {
            div.wrap {
                div.cta-in.reveal {
                    h2 { (tr.cta_title) }
                    p { (tr.cta_text) }
                    a.btn.play href="/download/setup" { (icons::download()) (tr.hero_cta_download) }
                }
            }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.title,
            css: &["home"],
            body_class: "home",
        },
        main,
    )
}

fn hero(p: &Page, latest: Option<&Release>, scene: &str) -> Markup {
    let tr = p.tr;
    let setup = latest.and_then(|r| r.asset(Kind::Setup));
    html! {
        section.hero {
            div.sky aria-hidden="true" { (PreEscaped(scene)) }
            div.wrap.hero-grid {
                div.hero-copy {
                    div.tags {
                        span { (icons::cube()) (tr.hero_tag_mc) }
                        span.sep { "·" }
                        span { (icons::rust()) (tr.hero_tag_rust) }
                    }
                    h1 { (tr.hero_title_1) br; span.accent { (tr.hero_title_2) } }
                    p.sub { (tr.hero_sub) }
                    div.actions {
                        a.btn.play href="/download/setup" { (icons::download()) (tr.hero_cta_download) }
                        a.btn.ghost href="/patch-notes" { (icons::notes()) (tr.hero_cta_notes) }
                    }
                    div.meta {
                        span { (icons::windows()) "Windows 10 / 11" }
                        @if let Some(r) = latest { span { (r.tag_name) } }
                        @if let Some(a) = setup { span { (format::size(tr, a.size)) } }
                        span { (tr.hero_free) }
                    }
                }
                aside.latest {
                    span.label { (tr.card_label) }
                    @match latest {
                        Some(r) => {
                            div.ver {
                                strong { (r.tag_name) }
                                span { (format::date(tr, r.published_at.as_deref())) }
                            }
                            @let points = markdown::highlights(r.notes(), 4);
                            @if points.is_empty() {
                                p.muted { (tr.card_no_notes) }
                            } @else {
                                span.small { (tr.card_whats_new) }
                                ul { @for pt in &points { li { (pt) } } }
                            }
                            a.more href=(format!("/patch-notes#{}", r.tag_name)) { (tr.card_more) " →" }
                        }
                        None => {
                            p.muted { (tr.card_unavailable) }
                            a.more href="/patch-notes" { (tr.card_more) " →" }
                        }
                    }
                }
            }
        }
    }
}

fn preview(p: &Page) -> Markup {
    let tr = p.tr;
    let shot = |name: &str| format!("/assets/screens/{}/{name}.png", p.lang.screens_dir());
    html! {
        section.sec #preview {
            div.wrap {
                div.head.reveal {
                    span.label { (tr.prev_label) }
                    h2 { (tr.prev_title) }
                    p { (tr.prev_text) }
                }
                div.gallery.reveal {
                    figure.big { img src=(shot("home")) alt=(tr.alt_home) width="1180" height="720" loading="lazy"; }
                    figure { img src=(shot("profiles")) alt=(tr.alt_profiles) width="1180" height="720" loading="lazy"; }
                    figure { img src=(shot("textures")) alt=(tr.alt_textures) width="1180" height="720" loading="lazy"; }
                }
                div.trio.reveal {
                    div { h3 { (icons::package()) (tr.f1_t) } p { (tr.f1_p) } }
                    div { h3 { (icons::cube()) (tr.f2_t) } p { (tr.f2_p) } }
                    div { h3 { (icons::brush()) (tr.f3_t) span.tag { (tr.experimental) } } p { (tr.f3_p) } }
                }
            }
        }
    }
}

fn card(icon: Markup, title: &str, text: &str) -> Markup {
    html! {
        div.card.reveal {
            span.ico { (icon) }
            h3 { (title) }
            p { (text) }
        }
    }
}

fn download(p: &Page, latest: Option<&Release>) -> Markup {
    let tr = p.tr;
    let meta = |kind: Kind| -> Option<String> {
        let r = latest?;
        let a = r.asset(kind)?;
        Some(format!("{} · {}", r.tag_name, format::size(tr, a.size)))
    };
    html! {
        (layout::fog_divider(p))
        section.sec #download {
            div.wrap {
                div.head.reveal {
                    span.label { (tr.dl_label) }
                    h2 { (tr.dl_title) }
                    p { (tr.dl_text) }
                }
                div.dl.reveal {
                    a.dl-card.main href="/download/setup" {
                        span.top { (icons::download()) (tr.inst_t) span.pill { (tr.recommended) } }
                        span.txt { (tr.inst_p) }
                        @if let Some(m) = meta(Kind::Setup) { span.m { (m) " · .exe" } }
                    }
                    a.dl-card href="/download/portable" {
                        span.top { (icons::archive()) (tr.port_t) }
                        span.txt { (tr.port_p) }
                        @if let Some(m) = meta(Kind::Portable) { span.m { (m) " · .zip" } }
                    }
                    a.dl-card href="/patch-notes" {
                        span.top { (icons::notes()) (tr.notes_t) }
                        span.txt { (tr.notes_p) }
                    }
                    a.dl-card href=(p.releases_url()) target="_blank" rel="noopener" {
                        span.top { (icons::github()) (tr.github_t) (icons::external()) }
                        span.txt { (tr.github_p) }
                    }
                }
            }
        }
    }
}

fn faq(p: &Page) -> Markup {
    let tr = p.tr;
    let qa = [
        (&tr.q1, &tr.a1),
        (&tr.q2, &tr.a2),
        (&tr.q3, &tr.a3),
        (&tr.q4, &tr.a4),
        (&tr.q5, &tr.a5),
    ];
    html! {
        section.sec #faq {
            div.wrap.faq {
                div.head.left.reveal {
                    span.label { (tr.faq_label) }
                    h2 { (tr.faq_title) }
                }
                div.qa.reveal {
                    @for (q, a) in qa {
                        details { summary { span { (q) } (icons::chevron()) } p { (a) } }
                    }
                }
            }
        }
    }
}
