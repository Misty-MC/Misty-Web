//! Documentation : page d'accueil (`/docs`) et pages (`/docs/<slug>`).

use super::icons;
use super::layout::{self, Head, Page};
use crate::docs::{DocPage, DocSet, Section};
use maud::{Markup, PreEscaped, html};

/// Dépôt du site, pour le lien « Modifier cette page ».
const WEB_REPO: &str = "Misty-MC/Misty-Web";

/// Accueil de la documentation : toutes les sections et leurs pages.
pub fn index(p: &Page, set: &DocSet) -> Markup {
    let tr = p.tr;
    let main = html! {
        section.docs-hero {
            div.wrap {
                span.label { (tr.docs_label) }
                h1 { (tr.docs_h1_1) " " span.accent { (tr.docs_h1_2) } }
                p.sub { (tr.docs_sub) }
                (search_box(p))
            }
        }
        section.wrap.docs-map {
            (untranslated(p, set))
            @for s in Section::ALL {
                @let pages: Vec<_> = set.in_section(s).collect();
                @if !pages.is_empty() {
                    div.map-sec.reveal {
                        h2 { span.n { (format!("{:02}", Section::ALL.iter().position(|x| *x == s).unwrap() + 1)) } (s.label(tr)) }
                        ul {
                            @for d in pages {
                                li data-search=(search_text(d)) data-slug=(d.slug) {
                                    a href=(format!("/docs/{}", d.slug)) {
                                        strong { (d.title) }
                                        span { (d.description) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p.docs-empty hidden { (tr.docs_no_results) }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.docs_title,
            description: &tr.docs_sub,
            jsonld: None,
            css: &["docs"],
            scripts: &[],
            body_class: "docs",
        },
        main,
    )
}

/// Une page de documentation.
pub fn page(p: &Page, set: &DocSet, idx: usize) -> Markup {
    let tr = p.tr;
    let d = &set.pages[idx];
    let prev = idx.checked_sub(1).map(|i| &set.pages[i]);
    let next = set.pages.get(idx + 1);
    let title = format!("{} – {}", d.title, tr.docs_title);
    let main = html! {
        div.wrap.docs-grid {
            aside.docs-side aria-label=(tr.docs_menu) {
                (search_box(p))
                (sidebar(p, set, &d.slug))
            }
            article.doc {
                details.docs-mobile {
                    summary { (icons::grid()) (tr.docs_menu) }
                    (sidebar(p, set, &d.slug))
                }
                nav.crumbs aria-label="Breadcrumb" {
                    a href="/docs" { (tr.docs_label) }
                    span { "/" }
                    span { (d.section.label(tr)) }
                }
                (untranslated(p, set))
                h1 { (d.title) }
                p.lead { (d.description) }
                div.md-doc { (PreEscaped(&d.html)) }
                nav.pager {
                    @if let Some(pv) = prev {
                        a.prev href=(format!("/docs/{}", pv.slug)) { span.small { "← " (tr.docs_prev) } strong { (pv.title) } }
                    } @else { span {} }
                    @if let Some(nx) = next {
                        a.next href=(format!("/docs/{}", nx.slug)) { span.small { (tr.docs_next) " →" } strong { (nx.title) } }
                    }
                }
                div.doc-foot {
                    a href=(format!("https://github.com/{WEB_REPO}/blob/main/{}", d.path)) target="_blank" rel="noopener" { (icons::github()) (tr.docs_edit) }
                    a href=(format!("https://github.com/{}/issues", p.repo)) target="_blank" rel="noopener" { (tr.docs_report) (icons::external()) }
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
            jsonld: Some(crate::seo::doc_jsonld(p.site_url, tr, d, p.lang)),
            css: &["docs"],
            scripts: &[],
            body_class: "docs",
        },
        main,
    )
}

/// Texte cherché par la recherche de la doc : titre, description et intertitres de la page.
fn search_text(d: &DocPage) -> String {
    let mut s = format!("{} {}", d.title, d.description);
    for t in &d.toc {
        s.push(' ');
        s.push_str(&t.text);
    }
    s.to_lowercase()
}

fn search_box(p: &Page) -> Markup {
    html! {
        label.docs-search {
            (icons::search())
            input type="search" placeholder=(p.tr.docs_search) aria-label=(p.tr.docs_search) autocomplete="off";
            kbd { "/" }
        }
    }
}

fn sidebar(p: &Page, set: &DocSet, current: &str) -> Markup {
    let tr = p.tr;
    html! {
        nav.docs-nav {
            @for s in Section::ALL {
                @let pages: Vec<_> = set.in_section(s).collect();
                @if !pages.is_empty() {
                    div.nav-sec {
                        span.small { (s.label(tr)) }
                        ul {
                            @for d in pages {
                                li data-search=(search_text(d)) data-slug=(d.slug) {
                                    a href=(format!("/docs/{}", d.slug)) aria-current=[(d.slug == current).then_some("page")] { (d.title) }
                                }
                            }
                        }
                    }
                }
            }
            p.docs-empty hidden { (tr.docs_no_results) }
        }
    }
}

/// Bandeau affiché quand la page s'affiche en anglais faute de traduction.
fn untranslated(p: &Page, set: &DocSet) -> Markup {
    html! {
        @if set.lang_dir != p.lang.code() && !p.tr.docs_untranslated.is_empty() {
            p.untranslated { (icons::globe()) (p.tr.docs_untranslated) }
        }
    }
}
