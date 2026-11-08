//! Boutique de cosmétiques : pour l'instant, on choisit son pseudo Minecraft
//! puis la page annonce l'ouverture prochaine.

use super::icons;
use super::layout::{self, Head, Page};
use maud::{Markup, html};

/// Ce que la page affiche.
pub enum ShopView<'a> {
    /// Formulaire du pseudo (avec la saisie précédente et, si besoin, l'erreur).
    Form { value: &'a str, invalid: bool },
    /// Pseudo choisi : « bientôt disponible ».
    Soon { player: &'a str },
}

/// Pseudo Minecraft valide : 3 à 16 caractères, lettres ASCII, chiffres et `_`.
pub fn valid_username(name: &str) -> bool {
    (3..=16).contains(&name.len()) && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

pub fn render(p: &Page, view: ShopView) -> Markup {
    let tr = p.tr;
    // Skin affiché dans la vitrine 3D : celui du joueur choisi, sinon Steve.
    let (skin, tag) = match &view {
        ShopView::Soon { player } => (*player, Some(*player)),
        ShopView::Form { value, .. } if valid_username(value) => (*value, Some(*value)),
        ShopView::Form { .. } => ("MHF_Steve", None),
    };
    let main = html! {
        section.hero.pn-hero.shop-hero {
            div.wrap.shop-grid {
                div.pn-copy {
                    div.tags { span { (icons::bag()) (tr.shop_label) } }
                    h1 { (tr.shop_h1_1) br; span.accent { (tr.shop_h1_2) } }
                    p.sub { (tr.shop_sub) }
                    @match view {
                        ShopView::Form { value, invalid } => { (form(p, value, invalid)) }
                        ShopView::Soon { player } => { (soon(p, player)) }
                    }
                }
                (stage(p, skin, tag))
            }
        }
        section.sec.shop-cats {
            div.wrap {
                div.cats {
                    (category(icons::cape(), &tr.shop_cat_capes, &tr.shop_cat_capes_p, &tr.shop_badge))
                    (category(icons::crown(), &tr.shop_cat_hats, &tr.shop_cat_hats_p, &tr.shop_badge))
                    (category(icons::wings(), &tr.shop_cat_wings, &tr.shop_cat_wings_p, &tr.shop_badge))
                    (category(icons::sparkles(), &tr.shop_cat_effects, &tr.shop_cat_effects_p, &tr.shop_badge))
                }
            }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.shop_title,
            description: &tr.shop_sub,
            jsonld: None,
            css: &["home", "patch-notes", "shop"],
            scripts: &["/vendor/skinview3d/skinview3d.bundle.js"],
            body_class: "shop",
        },
        main,
    )
}

/// Vitrine 3D : le skin du joueur (skinview3d) sur un cercle de lumière, avec un halo de lune,
/// des lucioles et des boutons pour essayer la cape et les élytres Misty.
/// Le skin est relayé par le serveur (`/skin/<pseudo>`) ; le pseudo est déjà validé.
fn stage(p: &Page, skin: &str, tag: Option<&str>) -> Markup {
    let tr = p.tr;
    html! {
        div.stage #stage data-skin=(format!("/skin/{skin}")) data-name=(tag.unwrap_or("")) {
            span.stage-label { (icons::sparkles()) (tr.shop_preview) }
            div.halo aria-hidden="true" {}
            div.circle aria-hidden="true" { span {} span {} span {} }
            div.motes aria-hidden="true" { @for _ in 0..10 { i {} } }
            canvas #skin3d role="img" aria-label=(tr.shop_preview) {}
            div.try role="group" aria-label=(tr.shop_preview) {
                button type="button" data-back="none" aria-pressed="true" { (tr.shop_try_none) }
                button type="button" data-back="cape" aria-pressed="false" { (icons::cape()) (tr.shop_try_cape) }
                button type="button" data-back="elytra" aria-pressed="false" { (icons::wings()) (tr.shop_try_elytra) }
            }
            p.drag { (tr.shop_drag) }
        }
    }
}

fn form(p: &Page, value: &str, invalid: bool) -> Markup {
    let tr = p.tr;
    html! {
        form.shop-form method="get" action="/shop" novalidate[invalid] {
            label for="player" { (tr.shop_username) }
            div.field.error[invalid] {
                span.prefix { (icons::user()) }
                input #player name="player" type="text" value=(value) placeholder=(tr.shop_placeholder)
                    minlength="3" maxlength="16" pattern="[A-Za-z0-9_]{3,16}" required
                    autocomplete="off" autocapitalize="off" spellcheck="false"
                    aria-invalid=[invalid.then_some("true")] aria-describedby="player-help";
                button.btn.play type="submit" { (tr.shop_continue) " →" }
            }
            @if invalid {
                p.help.err #player-help role="alert" { (tr.shop_invalid) }
            } @else {
                p.help #player-help { (tr.shop_hint) }
            }
        }
    }
}

fn soon(p: &Page, player: &str) -> Markup {
    let tr = p.tr;
    let (before, after) = tr
        .shop_hello
        .split_once("{name}")
        .unwrap_or((&tr.shop_hello, ""));
    html! {
        div.shop-card {
            img.head src=(format!("https://mc-heads.net/avatar/{player}/96")) alt=(player) width="72" height="72" loading="lazy";
            div {
                p.hello { (before) strong { (player) } (after) }
                span.soon { i.dot {} (tr.shop_soon) }
                p.muted { (tr.shop_soon_p) }
                a.link href="/shop?change=1" { (tr.shop_change) }
            }
        }
    }
}

fn category(icon: Markup, title: &str, text: &str, badge: &str) -> Markup {
    html! {
        div.cat.reveal {
            span.ico { (icon) }
            h3 { (title) span.badge { (badge) } }
            p { (text) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usernames() {
        for ok in ["Steve", "abc", "Misty_x_x", "A1234567890123_5"] {
            assert!(valid_username(ok), "{ok}");
        }
        for ko in [
            "",
            "ab",
            "trop_long_pseudo_17",
            "é_accent",
            "a b c",
            "<script>",
            "../x",
        ] {
            assert!(!valid_username(ko), "{ko}");
        }
    }
}
