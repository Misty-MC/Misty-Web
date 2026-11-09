//! Pages du captcha de vérification Discord (voir `crate::verify` et `crate::puzzle`) :
//! trois étapes (consentement, défi, vérifié) dans une seule carte, pilotée par `/js/captcha.js`.
//! La même carte sert dans le navigateur (`/verify`) et dans Discord (`/activity`).

use super::layout::{self, Head, Page};
use crate::puzzle;
use maud::{Markup, PreEscaped, html};

pub enum Etat {
    /// Puzzle prêt (lien valide)
    Puzzle { token: String, captcha: String, y: u32, server: Option<String> },
    LienInvalide,
    Indisponible,
    TropDEssais,
    DejaVerifie,
}

/// La carte : étapes, consentement, puzzle, fin. Les textes de fin sont dans des `data-*`.
fn carte(p: &Page, server: Option<&str>, data: Markup) -> Markup {
    let tr = p.tr;
    html! {
        div.cp #cp
            data-wrong=(tr.vf_wrong) data-off=(tr.vf_off_p) data-off-title=(tr.vf_off_title)
            data-many=(tr.vf_many_p) data-role=(tr.vf_role_p) data-role-title=(tr.vf_role_title)
            data-auth=(tr.vf_auth_p) data-auth-title=(tr.vf_auth_t)
            data-here=(tr.vf_here_p) data-here-title=(tr.vf_here_t) data-already=(tr.vf_already_p) data-already-title=(tr.vf_already_t) data-bad=(tr.vf_bad_p) data-bad-title=(tr.vf_bad_title)
            data-ok-title=(tr.vf_ok_title) data-ok=(tr.vf_ok_p) data-autoclose=(tr.vf_autoclose) data-start=(tr.vf_submit)
            data-piece-w=(puzzle::PIECE_W) data-w=(puzzle::W) data-h=(puzzle::H)
        {
            (data)
            div.cp-head {
                img.cp-logo src="/assets/icone.png" alt="" width="52" height="52";
                div {
                    h2 { (tr.vf_title_short) }
                    @if let Some(s) = server { p { (tr.vf_for) " " strong { (s) } } }
                }
            }
            ol.cp-steps {
                li #cp-s1 .on { span {} (tr.vf_step_1) }
                li #cp-s2 { span {} (tr.vf_step_2) }
                li #cp-s3 { span {} (tr.vf_step_3) }
            }
            // 1. Consentement
            section #cp-consent {
                p.cp-text { (tr.vf_consent_p) }
                button.btn.play #cp-start type="button" disabled { span #cp-start-label { (tr.vf_loading) } }
            }
            // 2. Le puzzle
            section #cp-challenge hidden {
                p.cp-text.center { (tr.vf_drag) }
                div.cp-stage #cp-stage {
                    img.cp-bg #cp-bg alt=(tr.vf_input) draggable="false";
                    img.cp-piece #cp-piece alt="" draggable="false";
                }
                p.vf-msg.bad #cp-err role="alert" hidden {}
                div.cp-slider #cp-slider {
                    div.cp-fill #cp-fill {}
                    span.cp-label { (tr.vf_slide) }
                    button.cp-knob #cp-knob type="button" aria-label=(tr.vf_slide) { (PreEscaped(r#"<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12h14m-6-6 6 6-6 6"/></svg>"#)) }
                }
                button.cp-another #cp-another type="button" { "↻ " (tr.vf_new) }
            }
            // 3. Fin (réussite ou erreur)
            section.vf-end #cp-end hidden {
                span.vf-badge #cp-badge {}
                h2 #cp-title {}
                p #cp-msg {}
                code.vf-detail #cp-detail hidden {}
                p.vf-hint #cp-auto hidden {}
                button.btn.ghost #cp-close type="button" hidden { (tr.vf_close) }
            }
            p.cp-privacy { (tr.vf_hint) }
        }
    }
}

fn head<'a>(p: &'a Page, body_class: &'a str) -> Head<'a> {
    Head {
        title: &p.tr.vf_title,
        description: &p.tr.vf_sub,
        jsonld: None,
        css: &["home", "patch-notes", "verify"],
        scripts: &[],
        body_class,
    }
}

/// `/verify` dans le navigateur.
pub fn render(p: &Page, etat: Etat) -> Markup {
    let tr = p.tr;
    let v = super::layout::ASSET_VERSION.as_str();
    let corps = match etat {
        Etat::Puzzle { token, captcha, y, server } => {
            let data = html! { div hidden #cp-data data-token=(token) data-captcha=(captcha) data-y=(y) {} };
            html! {
                (carte(p, server.as_deref(), data))
                script type="module" src=(format!("/js/verify-page.js?v={v}")) {}
            }
        }
        Etat::LienInvalide => fin(&tr.vf_bad_title, &tr.vf_bad_p),
        Etat::Indisponible => fin(&tr.vf_off_title, &tr.vf_off_p),
        Etat::TropDEssais => fin(&tr.vf_bad_title, &tr.vf_many_p),
        Etat::DejaVerifie => fin(&tr.vf_already_t, &tr.vf_already_p),
    };
    let main = html! { section.hero.pn-hero.vf-hero { div.wrap { div.pn-copy { (corps) } } } };
    layout::render(p, head(p, "verify-page"), main)
}

/// Erreur avant même le puzzle (lien expiré…).
fn fin(titre: &str, texte: &str) -> Markup {
    html! {
        div.cp {
            div.vf-end.bad {
                span.vf-badge.bad { "!" }
                h2 { (titre) }
                p { (texte) }
            }
        }
    }
}

/// `/activity` dans Discord : le SDK Discord identifie le membre, puis même carte.
pub fn activite(p: &Page, client_id: Option<&str>, server: Option<&str>) -> Markup {
    let v = super::layout::ASSET_VERSION.as_str();
    let data = html! { div hidden #cp-data data-client=[client_id] {} };
    let main = html! {
        section.hero.pn-hero.vf-hero { div.wrap { div.pn-copy {
            (carte(p, server, data))
            script type="module" src=(format!("/js/activity.js?v={v}")) {}
        } } }
    };
    layout::render(p, head(p, "verify-page activity"), main)
}
