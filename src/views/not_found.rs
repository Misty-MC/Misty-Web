//! Page 404.

use super::layout::{self, Head, Page};
use maud::{Markup, PreEscaped, html};

pub fn render(p: &Page, scene: &str) -> Markup {
    let tr = p.tr;
    let main = html! {
        section.hero.nf {
            div.sky aria-hidden="true" { (PreEscaped(scene)) }
            div.wrap { div.pn-copy {
                div.code { "404" }
                h1 { (tr.nf_title) }
                p.sub { (tr.nf_text) }
                div.actions { a.btn.play href="/" { (tr.nf_back) } }
            } }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.nf_title,
            css: &["home", "patch-notes"],
            body_class: "notes",
        },
        main,
    )
}
