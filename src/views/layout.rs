//! Squelette commun : <head>, barre de navigation et pied de page.

use super::icons;
use crate::i18n::{Lang, Tr};
use maud::{DOCTYPE, Markup, PreEscaped, html};
use std::sync::LazyLock;

/// Serveur Discord de Misty.
pub const DISCORD_URL: &str = "https://discord.gg/rdQJUgvUh3";

/// Ajouté aux adresses des CSS et du JS (`?v=`) : change à chaque démarrage du serveur,
/// donc à chaque déploiement, pour que les navigateurs ne gardent pas d'anciennes versions en cache.
pub static ASSET_VERSION: LazyLock<String> = LazyLock::new(|| {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
});

/// Ce dont toutes les pages ont besoin.
pub struct Page<'a> {
    pub lang: Lang,
    pub tr: &'static Tr,
    /// Chemin courant (sans paramètres), pour les liens de changement de langue.
    pub path: &'a str,
    /// Nombre de versions publiées, affiché à côté de « Patch notes ».
    pub release_count: Option<usize>,
    /// `owner/repo` du launcher sur GitHub.
    pub repo: &'a str,
    /// Bande de brume animée (SVG), réutilisée entre les sections.
    pub fog: &'a str,
    /// Paysage de fond (SVG), identique sur toutes les pages.
    pub scene: &'a str,
    /// Adresse publique du site, sans `/` final (balises canoniques, Open Graph, sitemap).
    pub site_url: &'a str,
}

impl Page<'_> {
    pub fn github_url(&self) -> String {
        format!("https://github.com/{}", self.repo)
    }
    pub fn releases_url(&self) -> String {
        format!("https://github.com/{}/releases", self.repo)
    }
}

pub struct Head<'a> {
    pub title: &'a str,
    /// Description de la page pour les moteurs de recherche et les aperçus de liens.
    pub description: &'a str,
    /// Données structurées JSON-LD (déjà sérialisées), pour les moteurs de recherche.
    pub jsonld: Option<String>,
    /// Feuilles de style propres à la page, en plus de base.css et layout.css.
    pub css: &'a [&'a str],
    /// Scripts propres à la page, chargés avant site.js.
    pub scripts: &'a [&'a str],
    pub body_class: &'a str,
}

pub fn render(p: &Page, head: Head, main: Markup) -> Markup {
    let tr = p.tr;
    html! {
        (DOCTYPE)
        html lang=(p.lang.code()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (head.title) }
                meta name="description" content=(head.description);
                meta name="theme-color" content="#0a0f1d";
                meta name="robots" content="index, follow, max-image-preview:large";
                meta name="application-name" content="Misty MC";
                // Une URL par langue (?lang=xx) : la canonique pointe sur la langue affichée,
                // et les variantes hreflang annoncent les autres à Google.
                @let base = format!("{}{}", p.site_url, p.path);
                @let canonical = format!("{base}?lang={}", p.lang.code());
                link rel="canonical" href=(canonical);
                @for l in Lang::ALL {
                    link rel="alternate" hreflang=(l.code()) href=(format!("{base}?lang={}", l.code()));
                }
                link rel="alternate" hreflang="x-default" href=(base);
                // Aperçus de liens (Discord, réseaux sociaux…)
                @let image = format!("{}/assets/screens/en/home.png?v={}", p.site_url, ASSET_VERSION.as_str());
                meta property="og:type" content="website";
                meta property="og:site_name" content="Misty MC";
                meta property="og:title" content=(head.title);
                meta property="og:description" content=(head.description);
                meta property="og:url" content=(canonical);
                meta property="og:image" content=(image);
                meta property="og:image:width" content="1180";
                meta property="og:image:height" content="720";
                meta property="og:locale" content=(tr.og_locale);
                meta name="twitter:card" content="summary_large_image";
                meta name="twitter:title" content=(head.title);
                meta name="twitter:description" content=(head.description);
                meta name="twitter:image" content=(image);
                link rel="icon" href="/assets/icone.png";
                link rel="apple-touch-icon" href="/assets/icone.png";
                @if let Some(json) = &head.jsonld {
                    script type="application/ld+json" { (PreEscaped(json)) }
                }
                link rel="preload" href="/assets/Silkscreen-Regular.ttf" as="font" type="font/ttf" crossorigin;
                @let v = ASSET_VERSION.as_str();
                link rel="stylesheet" href=(format!("/css/base.css?v={v}"));
                link rel="stylesheet" href=(format!("/css/layout.css?v={v}"));
                @for css in head.css { link rel="stylesheet" href=(format!("/css/{css}.css?v={v}")); }
                script { (PreEscaped("document.documentElement.classList.add('js')")) }
                @for js in head.scripts { script src=(format!("{js}?v={v}")) defer {} }
                script src=(format!("/js/site.js?v={v}")) defer {}
            }
            body class=(head.body_class) {
                // Voile de brume qui dérive derrière tout le site.
                div.mist aria-hidden="true" { div.m1 {} div.m2 {} div.m3 {} }
                // Ciel et paysage : le même fond en haut de chaque page.
                div.page-sky aria-hidden="true" { (PreEscaped(p.scene)) }
                (nav(p))
                main #main { (main) }
                (fog_divider(p))
                (footer(p))
            }
        }
    }
}

/// Bande de brume pixel qui dérive, posée entre deux sections.
pub fn fog_divider(p: &Page) -> Markup {
    html! { div.fog-divider aria-hidden="true" { (PreEscaped(p.fog)) } }
}

fn nav(p: &Page) -> Markup {
    let tr = p.tr;
    let on_notes = p.path.starts_with("/patch-notes");
    html! {
        header.nav #top {
            div.wrap.nav-in {
                a.brand href="/" { img src="/assets/icone.png" alt="" width="30" height="30"; span { "Misty" } }
                nav.links aria-label="Navigation" {
                    a href="/#features" { (tr.nav_features) }
                    a href="/#loaders" { (tr.nav_loaders) }
                    a href="/#faq" { (tr.nav_faq) }
                    a href="/docs" aria-current=[p.path.starts_with("/docs").then_some("page")] { (tr.nav_docs) }
                    a href="/shop" aria-current=[p.path.starts_with("/shop").then_some("page")] { (tr.nav_shop) }
                    a href="/patch-notes" aria-current=[on_notes.then_some("page")] {
                        (tr.nav_patch_notes)
                        @if let Some(n) = p.release_count { span.count { (n) } }
                    }
                }
                div.nav-right {
                    a.gh.dc href=(DISCORD_URL) target="_blank" rel="noopener" aria-label="Discord" title="Discord" { (icons::discord()) }
                    a.gh href=(p.github_url()) target="_blank" rel="noopener" aria-label="GitHub" { (icons::github()) span { "GitHub" } }
                    (lang_menu(p))
                    a.btn.play.sm href="/#download" { (icons::download()) span { (tr.nav_download) } }
                    (mobile_menu(p))
                }
            }
        }
    }
}

/// Menu de langue sans JavaScript : un <details> dont les liens rechargent la page avec `?lang=`.
/// Menu des pages pour les petits écrans (la barre de liens y est masquée).
fn mobile_menu(p: &Page) -> Markup {
    let tr = p.tr;
    let here = |path: &str| {
        (p.path == path || (path != "/" && p.path.starts_with(path))).then_some("page")
    };
    html! {
        details.burger {
            summary aria-label="Menu" { (icons::menu()) }
            nav.menu aria-label="Navigation" {
                a href="/#features" { (tr.nav_features) }
                a href="/#loaders" { (tr.nav_loaders) }
                a href="/#faq" { (tr.nav_faq) }
                a href="/docs" aria-current=[here("/docs")] { (tr.nav_docs) }
                a href="/shop" aria-current=[here("/shop")] { (tr.nav_shop) }
                a href="/patch-notes" aria-current=[here("/patch-notes")] {
                    (tr.nav_patch_notes)
                    @if let Some(n) = p.release_count { span.count { (n) } }
                }
                a href="/status" aria-current=[here("/status")] { (tr.nav_status) }
                a href=(DISCORD_URL) target="_blank" rel="noopener" { "Discord" (icons::external()) }
                a href=(p.github_url()) target="_blank" rel="noopener" { "GitHub" (icons::external()) }
            }
        }
    }
}

fn lang_menu(p: &Page) -> Markup {
    html! {
        details.langdd {
            summary aria-label=(p.tr.nav_language) { (icons::globe()) span { (p.lang.code().to_uppercase()) } }
            div.menu {
                @for l in Lang::ALL {
                    a href=(format!("{}?lang={}", p.path, l.code())) hreflang=(l.code()) aria-current=[(l == p.lang).then_some("true")] {
                        (l.tr().lang_name) span { (l.code().to_uppercase()) }
                    }
                }
            }
        }
    }
}

fn footer(p: &Page) -> Markup {
    let tr = p.tr;
    html! {
        footer.foot {
            div.wrap {
                div.ftop {
                    div {
                        a.brand href="/" { img src="/assets/icone.png" alt="" width="30" height="30"; span { "Misty" } }
                        p.tag { (tr.ft_tag) }
                    }
                    p.disclaimer { (tr.ft_disclaimer) }
                }
                div.fcols {
                    div {
                        h2 { (tr.ft_nav) }
                        ul {
                            li { a href="/#features" { (tr.nav_features) } }
                            li { a href="/#loaders" { (tr.nav_loaders) } }
                            li { a href="/#faq" { (tr.nav_faq) } }
                            li { a href="/docs" { (tr.docs_label) } }
                            li { a href="/shop" { (tr.nav_shop) } }
                            li { a href="/patch-notes" { (tr.nav_patch_notes) } }
                            li { a href="/status" { (tr.nav_status) } }
                        }
                    }
                    div {
                        h2 { (tr.nav_download) }
                        ul {
                            li { a href="/download/setup" { (tr.inst_t) } }
                            li { a href="/download/portable" { (tr.port_t) } }
                            li { a href=(p.releases_url()) target="_blank" rel="noopener" { (tr.github_t) (icons::external()) } }
                            li { a href=(p.github_url()) target="_blank" rel="noopener" { "GitHub" (icons::external()) } }
                        }
                    }
                    div {
                        h2 { (tr.nav_language) }
                        div.flangs {
                            @for l in Lang::ALL {
                                a href=(format!("{}?lang={}", p.path, l.code())) hreflang=(l.code()) aria-current=[(l == p.lang).then_some("true")] { (l.code().to_uppercase()) }
                            }
                        }
                    }
                    div.about {
                        h2 { (tr.ft_about_t) }
                        p { (tr.ft_about_p) }
                        a.discord-cta href=(DISCORD_URL) target="_blank" rel="noopener" {
                            (icons::discord())
                            span { strong { (tr.discord_join) } small { (tr.discord_sub) } }
                            (icons::external())
                        }
                    }
                }
                div.fbottom {
                    span.made { (icons::rust()) (tr.ft_made) }
                    span.fb-right {
                        a href=(DISCORD_URL) target="_blank" rel="noopener" { (icons::discord()) "Discord" }
                        a href=(p.github_url()) target="_blank" rel="noopener" { (icons::github()) (p.repo) }
                        a.totop href="#top" { (tr.ft_top) (icons::arrow_up()) }
                    }
                }
            }
            div.giant aria-hidden="true" { "MISTY" }
        }
    }
}
