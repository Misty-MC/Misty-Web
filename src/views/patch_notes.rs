//! Page des patch notes : toutes les versions publiées, filtrables, dépliables.

use super::icons;
use super::layout::{self, Head, Page};
use crate::format;
use crate::github::{self, Release};
use crate::markdown;
use maud::{Markup, PreEscaped, html};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Stable,
    Pre,
}

impl Filter {
    pub fn from_query(q: Option<&str>) -> Filter {
        match q {
            Some("stable") => Filter::Stable,
            Some("pre") => Filter::Pre,
            _ => Filter::All,
        }
    }
    fn slug(self) -> &'static str {
        match self {
            Filter::All => "all",
            Filter::Stable => "stable",
            Filter::Pre => "pre",
        }
    }
    fn keeps(self, r: &Release) -> bool {
        match self {
            Filter::All => true,
            Filter::Stable => !r.prerelease,
            Filter::Pre => r.prerelease,
        }
    }
}

pub fn render(p: &Page, releases: Option<&[Release]>, filter: Filter) -> Markup {
    let tr = p.tr;
    let main = html! {
        section.hero.pn-hero {
            div.wrap { div.pn-copy {
                div.tags {
                    span { (tr.pn_label) }
                    span.sep { "·" }
                    a href=(p.github_url()) target="_blank" rel="noopener" { (icons::github()) (p.repo) }
                }
                h1 { (tr.pn_h1_1) br; span.accent { (tr.pn_h1_2) } }
                p.sub { (tr.pn_sub) }
                div.actions {
                    a.btn.play href="#versions" { (icons::notes()) (tr.pn_browse) }
                    a.btn.ghost href=(p.releases_url()) target="_blank" rel="noopener" { (icons::github()) (tr.pn_all_github) }
                }
            } }
        }
        section.sec #versions {
            div.wrap.versions {
                @match releases {
                    None => { p.empty { (tr.pn_unavailable) } }
                    Some(list) => { (list_view(p, list, filter)) }
                }
            }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.pn_title,
            css: &["home", "patch-notes"],
            body_class: "notes",
        },
        main,
    )
}

fn list_view(p: &Page, list: &[Release], filter: Filter) -> Markup {
    let tr = p.tr;
    let latest_tag = github::latest(list).map(|r| r.tag_name.as_str());
    let stable = list.iter().filter(|r| !r.prerelease).count();
    let chips = [
        (Filter::All, &tr.pn_filter_all, list.len()),
        (Filter::Stable, &tr.pn_filter_stable, stable),
        (Filter::Pre, &tr.pn_filter_pre, list.len() - stable),
    ];
    let shown: Vec<&Release> = list.iter().filter(|r| filter.keeps(r)).collect();
    html! {
        div.toolbar {
            nav.chips aria-label=(tr.pn_label) {
                @for (f, label, n) in chips {
                    a href=(format!("/patch-notes?filter={}#versions", f.slug())) aria-current=[(f == filter).then_some("true")] {
                        (label) span { (n) }
                    }
                }
            }
        }
        @if shown.is_empty() {
            p.empty { (tr.pn_empty) }
        }
        div.rels {
            @for (i, r) in shown.iter().enumerate() {
                @let is_latest = Some(r.tag_name.as_str()) == latest_tag;
                details.rel id=(r.tag_name) open[i == 0] {
                    summary {
                        span.dot.on[is_latest] {}
                        span.badge.latest[is_latest].pre[r.prerelease] {
                            @if is_latest { (tr.pn_badge_latest) }
                            @else if r.prerelease { (tr.pn_badge_pre) }
                            @else { (tr.pn_badge_stable) }
                        }
                        span.rt {
                            strong { (r.title()) }
                            span.rm {
                                (format::date(tr, r.published_at.as_deref()))
                                @if r.title() != r.tag_name { " · " code { (r.tag_name) } }
                            }
                        }
                        (icons::chevron())
                    }
                    div.rb {
                        div.md {
                            @if r.notes().is_empty() { p { (tr.card_no_notes) } }
                            @else { (PreEscaped(markdown::render(r.notes()))) }
                        }
                        @if !r.assets.is_empty() {
                            div.files {
                                span.small { (tr.pn_files) }
                                @for a in &r.assets {
                                    a.file href=(a.browser_download_url) {
                                        (icons::download())
                                        span { (a.name) }
                                        span.sz { (format::size(tr, a.size)) }
                                    }
                                }
                            }
                        }
                        a.link href=(r.html_url) target="_blank" rel="noopener" { (tr.pn_view_github) (icons::external()) }
                    }
                }
            }
        }
    }
}
