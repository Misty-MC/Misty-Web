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
    // Le personnage qui se promène dans la forêt : le skin du joueur choisi, sinon Steve.
    let (skin, tag) = match &view {
        ShopView::Soon { player } => (*player, Some(*player)),
        ShopView::Form { value, .. } if valid_username(value) => (*value, Some(*value)),
        ShopView::Form { .. } => ("MHF_Steve", None),
    };
    let main = html! {
        (walker(skin, tag))
        section.hero.pn-hero.shop-hero {
            div.wrap { div.pn-copy {
                div.tags { span { (icons::bag()) (tr.shop_label) } }
                h1 { (tr.shop_h1_1) br; span.accent { (tr.shop_h1_2) } }
                p.sub { (tr.shop_sub) }
                @match view {
                    ShopView::Form { value, invalid } => { (form(p, value, invalid)) }
                    ShopView::Soon { player } => { (soon(p, player)) }
                }
            } }
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
            body_class: "shop",
        },
        main,
    )
}

/// Personnage en pixel art construit avec les faces du skin (vue de profil) : il marche dans
/// la forêt du fond, fait demi-tour au bout et saute quand on clique dessus.
/// Le skin vient de mc-heads.net ; le nom est déjà validé, il ne contient que [A-Za-z0-9_].
fn walker(skin: &str, tag: Option<&str>) -> Markup {
    let style = format!("--skin:url('https://mc-heads.net/skin/{skin}')");
    html! {
        div.walker-track aria-hidden="true" {
            div.walker #walker style=(style) {
                span.nametag hidden[tag.is_none()] { (tag.unwrap_or("")) }
                div.flip {
                    div.figure {
                        div.part.arm.back {}
                        div.part.leg.back {}
                        div.part.body {}
                        div.part.leg.front {}
                        div.part.arm.front {}
                        div.part.head { div.hat {} }
                    }
                }
                span.shadow {}
            }
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
