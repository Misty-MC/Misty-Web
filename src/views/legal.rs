//! Pages légales : mentions légales, confidentialité, conditions d'utilisation.

use super::icons;
use super::layout::{self, Head, Page};
use crate::legal::{Kind, LegalPage};
use maud::{Markup, PreEscaped, html};

/// Dépôt du site, pour le lien « Modifier cette page ».
const WEB_REPO: &str = "Misty-MC/Misty-Web";

pub fn render(p: &Page, d: &LegalPage) -> Markup {
    let tr = p.tr;
    let title = format!("{} – Misty MC", d.title);
    let main = html! {
        div.wrap.docs-grid {
            aside.docs-side aria-label=(tr.ft_legal_t) {
                (menu(p, d.kind))
            }
            article.doc {
                details.docs-mobile {
                    summary { (icons::grid()) (tr.ft_legal_t) }
                    (menu(p, d.kind))
                }
                nav.crumbs aria-label="Breadcrumb" {
                    a href="/" { "Misty" }
                    span { "/" }
                    span { (tr.ft_legal_t) }
                }
                @if d.lang_dir != p.lang.code() && !tr.legal_untranslated.is_empty() {
                    p.untranslated { (icons::globe()) (tr.legal_untranslated) }
                }
                h1 { (d.title) }
                p.lead {
                    (d.description)
                    br;
                    small { (tr.legal_updated) " " (crate::format::jour(tr, d.updated, true)) }
                }
                div.md-doc { (PreEscaped(&d.html)) }
                div.doc-foot {
                    a href=(format!("https://github.com/{WEB_REPO}/blob/main/{}", d.path)) target="_blank" rel="noopener" { (icons::github()) (tr.docs_edit) }
                }
            }
            @if !d.toc.is_empty() {
                aside.docs-toc {
                    span.small { (tr.docs_on_page) }
                    ul {
                        @for t in &d.toc {
                            li.lvl3[t.level == 3] { a href=(format!("#{}", t.id)) { (t.text) } }
                        }
                    }
                }
            }
        }
    };
    layout::render(
        p,
        Head {
            title: &title,
            description: &d.description,
            jsonld: None,
            css: &["docs"],
            scripts: &[],
            body_class: "docs",
        },
        main,
    )
}

fn menu(p: &Page, current: Kind) -> Markup {
    html! {
        nav.docs-nav {
            div.nav-sec {
                span.small { (p.tr.ft_legal_t) }
                ul {
                    @for k in Kind::ALL {
                        li { a href=(k.path()) aria-current=[(k == current).then_some("page")] { (k.label(p.tr)) } }
                    }
                }
            }
        }
    }
}
