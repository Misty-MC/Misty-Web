//! Page du captcha de vérification Discord (voir `crate::verify`).

use super::layout::{self, Head, Page};
use maud::{Markup, html};

pub enum Etat<'a> {
    /// Captcha à résoudre (`erreur` : la réponse précédente était fausse)
    Captcha { token: &'a str, defi: &'a str, erreur: bool },
    Reussi,
    LienInvalide,
    Indisponible,
    TropDEssais,
    EchecRole,
}

pub fn render(p: &Page, etat: Etat) -> Markup {
    let tr = p.tr;
    let carte = match etat {
        Etat::Captcha { token, defi, erreur } => html! {
            form.vf-card method="post" action=(format!("/verify?lang={}", p.lang.code())) autocomplete="off" {
                img.vf-img src=(format!("/verify/captcha/{defi}.svg")) width="330" height="100" alt=(tr.vf_input);
                @if erreur { p.vf-msg.bad role="alert" { (tr.vf_wrong) } }
                input type="hidden" name="t" value=(token);
                input type="hidden" name="id" value=(defi);
                // piège : invisible pour les humains, rempli par les robots naïfs
                div.vf-trap aria-hidden="true" { input type="text" name="website" tabindex="-1" autocomplete="off"; }
                label.vf-field {
                    span { (tr.vf_input) }
                    input type="text" name="code" required maxlength="12" autocapitalize="characters" spellcheck="false" autofocus inputmode="text";
                }
                div.vf-actions {
                    button.btn.play type="submit" { (tr.vf_submit) }
                    a.btn.ghost href=(format!("/verify?t={token}&lang={}", p.lang.code())) { (tr.vf_new) }
                }
                p.vf-hint { (tr.vf_hint) }
            }
        },
        Etat::Reussi => fin("ok", &tr.vf_ok_title, &tr.vf_ok_p),
        Etat::LienInvalide => fin("bad", &tr.vf_bad_title, &tr.vf_bad_p),
        Etat::Indisponible => fin("bad", &tr.vf_off_title, &tr.vf_off_p),
        Etat::TropDEssais => fin("bad", &tr.vf_bad_title, &tr.vf_many_p),
        Etat::EchecRole => fin("bad", &tr.vf_role_title, &tr.vf_role_p),
    };
    let main = html! {
        section.hero.pn-hero.vf-hero {
            div.wrap { div.pn-copy {
                div.tags { span { (tr.vf_label) } }
                h1 { (tr.vf_h1_1) " " span.accent { (tr.vf_h1_2) } }
                p.sub { (tr.vf_sub) }
                (carte)
            } }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.vf_title,
            description: &tr.vf_sub,
            jsonld: None,
            css: &["home", "patch-notes", "verify"],
            scripts: &[],
            body_class: "verify-page",
        },
        main,
    )
}

fn fin(classe: &str, titre: &str, texte: &str) -> Markup {
    html! {
        div.vf-card.vf-end.(classe) {
            span.vf-badge.(classe) { @if classe == "ok" { "✓" } @else { "!" } }
            h2 { (titre) }
            p { (texte) }
            a.btn.ghost href="discord://-/channels/@me" { "Discord" }
        }
    }
}

/// Page ouverte dans Discord (Activité) : tout se passe ici, sans changer de page, pour garder la
/// connexion au SDK Discord (`/js/activity.js`).
pub fn activite(p: &Page, client_id: Option<&str>) -> Markup {
    let tr = p.tr;
    let v = super::layout::ASSET_VERSION.as_str();
    let main = html! {
        section.hero.pn-hero.vf-hero {
            div.wrap { div.pn-copy {
                div.tags { span { (tr.vf_label) } }
                h1 { (tr.vf_h1_1) " " span.accent { (tr.vf_h1_2) } }
                p.sub { (tr.vf_sub) }
                div #vf-app.vf-card
                    data-client=[client_id]
                    data-wrong=(tr.vf_wrong) data-off=(tr.vf_off_p) data-many=(tr.vf_many_p)
                    data-role=(tr.vf_role_p) data-auth=(tr.vf_auth_p) data-bad=(tr.vf_bad_p)
                    data-bad-title=(tr.vf_bad_title) data-off-title=(tr.vf_off_title) data-role-title=(tr.vf_role_title)
                {
                    p.vf-wait #vf-wait { span.vf-spin {} (tr.vf_loading) }
                    form #vf-form hidden autocomplete="off" {
                        img.vf-img #vf-img width="330" height="100" alt=(tr.vf_input);
                        p.vf-msg.bad #vf-err role="alert" hidden {}
                        label.vf-field {
                            span { (tr.vf_input) }
                            input #vf-code type="text" required maxlength="12" autocapitalize="characters" spellcheck="false";
                        }
                        div.vf-actions {
                            button.btn.play type="submit" { (tr.vf_submit) }
                            button.btn.ghost #vf-new type="button" { (tr.vf_new) }
                        }
                        p.vf-hint { (tr.vf_hint) }
                    }
                    div.vf-end #vf-end hidden {
                        span.vf-badge #vf-badge {}
                        h2 #vf-title {}
                        p #vf-text {}
                        button.btn.ghost #vf-close type="button" { (tr.vf_close) }
                    }
                    div hidden #vf-ok data-title=(tr.vf_ok_title) data-text=(tr.vf_ok_p) {}
                }
                script type="module" src=(format!("/js/activity.js?v={v}")) {}
            } }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.vf_title,
            description: &tr.vf_sub,
            jsonld: None,
            css: &["home", "patch-notes", "verify"],
            scripts: &[],
            body_class: "verify-page activity",
        },
        main,
    )
}
