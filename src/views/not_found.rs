//! Page 404.

use super::layout::{self, Head, Page};
use maud::{Markup, html};

pub fn render(p: &Page) -> Markup {
    let tr = p.tr;
    let main = html! {
        section.hero.nf {
            div.wrap { div.pn-copy {
                div.code { "404" }
                h1 { (tr.nf_title.split(" – ").next().unwrap_or(&tr.nf_title)) }
                p.sub { (tr.nf_text) }
                div.actions { a.btn.play href="/" { (tr.nf_back) } }
            } }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.nf_title,
            description: &tr.nf_text,
            jsonld: None,
            css: &["home", "patch-notes"],
            scripts: &[],
            body_class: "notes",
        },
        main,
    )
}
