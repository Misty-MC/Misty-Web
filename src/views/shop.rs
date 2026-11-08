//! Boutique de cosmétiques.
//!
//! - Sans pseudo : formulaire, avec Steve en 3D à côté (aperçu en direct pendant la saisie).
//! - Avec un pseudo : le studio. À gauche, le skin du joueur en 3D (vues face / profil / dos,
//!   animations, sa vraie cape, élytres) ; à droite, sa fiche et les catégories de
//!   cosmétiques, toutes « bientôt disponibles », avec la cape et les élytres Misty à essayer.

use super::icons;
use super::layout::{self, Head, Page};
use crate::i18n::Tr;
use maud::{Markup, html};

/// Ce que la page affiche.
pub enum ShopView<'a> {
    /// Formulaire du pseudo (avec la saisie précédente et, si besoin, l'erreur).
    Form { value: &'a str, invalid: bool },
    /// Le studio du joueur choisi.
    Studio { player: &'a str },
}

/// Pseudo Minecraft valide : 3 à 16 caractères, lettres ASCII, chiffres et `_`.
pub fn valid_username(name: &str) -> bool {
    (3..=16).contains(&name.len()) && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Catégories de cosmétiques (inspirées des grandes boutiques de cosmétiques Minecraft).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Category {
    Capes,
    Wings,
    Headwear,
    Bodywear,
    Auras,
    Pets,
    Emotes,
    Bundles,
}

impl Category {
    const ALL: [Category; 8] = [
        Category::Capes,
        Category::Wings,
        Category::Headwear,
        Category::Bodywear,
        Category::Auras,
        Category::Pets,
        Category::Emotes,
        Category::Bundles,
    ];

    fn slug(self) -> &'static str {
        match self {
            Category::Capes => "capes",
            Category::Wings => "wings",
            Category::Headwear => "headwear",
            Category::Bodywear => "bodywear",
            Category::Auras => "auras",
            Category::Pets => "pets",
            Category::Emotes => "emotes",
            Category::Bundles => "bundles",
        }
    }

    fn text(self, tr: &Tr) -> (&str, &str) {
        match self {
            Category::Capes => (&tr.shop_cat_capes, &tr.shop_cat_capes_p),
            Category::Wings => (&tr.shop_cat_wings, &tr.shop_cat_wings_p),
            Category::Headwear => (&tr.shop_cat_hats, &tr.shop_cat_hats_p),
            Category::Bodywear => (&tr.shop_cat_bodywear, &tr.shop_cat_bodywear_p),
            Category::Auras => (&tr.shop_cat_effects, &tr.shop_cat_effects_p),
            Category::Pets => (&tr.shop_cat_pets, &tr.shop_cat_pets_p),
            Category::Emotes => (&tr.shop_cat_emotes, &tr.shop_cat_emotes_p),
            Category::Bundles => (&tr.shop_cat_bundles, &tr.shop_cat_bundles_p),
        }
    }

    fn icon(self) -> Markup {
        match self {
            Category::Capes => icons::cape(),
            Category::Wings => icons::wings(),
            Category::Headwear => icons::crown(),
            Category::Bodywear => icons::shirt(),
            Category::Auras => icons::sparkles(),
            Category::Pets => icons::paw(),
            Category::Emotes => icons::smile(),
            Category::Bundles => icons::package(),
        }
    }
}

pub fn render(p: &Page, view: ShopView) -> Markup {
    let tr = p.tr;
    let main = match view {
        ShopView::Form { value, invalid } => {
            let preview = valid_username(value).then_some(value);
            html! {
                section.hero.pn-hero.shop-hero {
                    div.wrap.shop-grid {
                        div.pn-copy {
                            div.tags { span { (icons::bag()) (tr.shop_label) } }
                            h1 { (tr.shop_h1_1) br; span.accent { (tr.shop_h1_2) } }
                            p.sub { (tr.shop_sub) }
                            (form(p, value, invalid))
                        }
                        (stage(p, preview.unwrap_or("MHF_Steve"), preview, false))
                    }
                }
            }
        }
        ShopView::Studio { player } => html! {
            section.studio {
                div.wrap {
                    nav.crumbs aria-label="Breadcrumb" {
                        a href="/shop" { (icons::bag()) (tr.shop_label) }
                        span { "/" }
                        span { (player) }
                    }
                    div.studio-grid {
                        (stage(p, player, Some(player), true))
                        (locker(p, player))
                    }
                }
            }
        },
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

/// Aperçu 3D (skinview3d) : vues, animations, cape et élytres.
/// Le skin et la cape sont relayés par le serveur (`/skin/…`, `/cape/…`, `/player/…`).
fn stage(p: &Page, skin: &str, name: Option<&str>, studio: bool) -> Markup {
    let tr = p.tr;
    let anims = [
        ("idle", &tr.shop_anim_idle),
        ("walk", &tr.shop_anim_walk),
        ("run", &tr.shop_anim_run),
        ("fly", &tr.shop_anim_fly),
        ("wave", &tr.shop_anim_wave),
        ("crouch", &tr.shop_anim_crouch),
    ];
    html! {
        div.stage.studio-stage[studio] #stage data-skin=(skin) data-name=(name.unwrap_or("")) {
            div.stage-top {
                div.seg role="group" aria-label=(tr.shop_view) {
                    button type="button" data-view="front" aria-pressed="true" { (tr.shop_view_front) }
                    button type="button" data-view="side" aria-pressed="false" { (tr.shop_view_side) }
                    button type="button" data-view="back" aria-pressed="false" { (tr.shop_view_back) }
                }
                button.toggle type="button" data-toggle="rotate" aria-pressed="true" { (icons::refresh()) span { (tr.shop_rotate) } }
            }
            div.halo aria-hidden="true" {}
            div.circle aria-hidden="true" { span {} span {} span {} }
            div.motes aria-hidden="true" { @for _ in 0..10 { i {} } }
            canvas #skin3d role="img" aria-label=(tr.shop_preview) {}
            div.stage-bar {
                div.seg.anims role="group" aria-label=(tr.shop_anim) {
                    @for (key, label) in anims {
                        button type="button" data-anim=(key) aria-pressed=(if key == "walk" { "true" } else { "false" }) { (label) }
                    }
                }
                div.toggles {
                    button.toggle type="button" data-toggle="cape" aria-pressed="false" disabled title=(tr.shop_no_cape) { (icons::cape()) span { (tr.shop_own_cape) } }
                    button.toggle type="button" data-toggle="elytra" aria-pressed="false" { (icons::wings()) span { (tr.shop_elytra) } }
                }
            }
            p.drag { (tr.shop_drag) }
        }
    }
}

/// Fiche du joueur et catégories de cosmétiques.
fn locker(p: &Page, player: &str) -> Markup {
    let tr = p.tr;
    html! {
        aside.locker {
            div.pcard {
                img.head src=(format!("https://mc-heads.net/avatar/{player}/96")) alt="" width="64" height="64";
                div.pinfo {
                    h1 #pname { (player) }
                    div.pmeta {
                        span.chip #pmodel data-classic=(tr.shop_model_classic) data-slim=(tr.shop_model_slim) { (tr.shop_model_classic) }
                    }
                    button.uuid #puuid type="button" title=(tr.shop_copy_uuid) data-copied=(tr.shop_copied) hidden {
                        code {} (icons::copy())
                    }
                    p.pmissing #pmissing hidden { (tr.shop_not_found) }
                }
            }
            div.plinks {
                a href=(format!("https://namemc.com/profile/{player}")) target="_blank" rel="noopener" { (tr.shop_namemc) (icons::external()) }
                a href="/shop?change=1" { (tr.shop_change) }
            }

            h2 { (tr.shop_cosmetics_t) }
            p.muted { (tr.shop_cosmetics_p) }
            div.cat-tabs role="tablist" aria-label=(tr.shop_cosmetics_t) {
                @for (i, c) in Category::ALL.iter().enumerate() {
                    button type="button" role="tab" id=(format!("tab-{}", c.slug())) aria-controls=(format!("cat-{}", c.slug()))
                        aria-selected=(if i == 0 { "true" } else { "false" }) data-cat=(c.slug()) {
                        (c.icon()) span { (c.text(tr).0) }
                    }
                }
            }
            @for (i, c) in Category::ALL.iter().enumerate() {
                @let (title, text) = c.text(tr);
                div.cat-panel role="tabpanel" id=(format!("cat-{}", c.slug())) aria-labelledby=(format!("tab-{}", c.slug())) hidden[i != 0] {
                    div.cat-head { h3 { (c.icon()) (title) } span.badge { (tr.shop_soon) } }
                    p { (text) }
                    @let item = match c {
                        Category::Capes => Some(("cape", &tr.shop_item_cape)),
                        Category::Wings => Some(("elytra", &tr.shop_item_elytra)),
                        _ => None,
                    };
                    div.slots {
                        // Première case : l'article Misty à essayer, s'il y en a un dans cette catégorie.
                        @if let Some((back, name)) = item {
                            div.slot.item {
                                canvas.thumb data-thumb=(back) width="60" height="96" aria-hidden="true" {}
                                strong { (name) }
                                button.btn.play.sm.equip type="button" data-back=(back) aria-pressed="false"
                                    data-equip=(tr.shop_equip) data-unequip=(tr.shop_unequip) { (tr.shop_equip) }
                            }
                        }
                        @for _ in 0..(if item.is_some() { 5 } else { 6 }) {
                            div.slot { (c.icon()) span { (tr.shop_badge) } }
                        }
                    }
                    @if item.is_some() {
                        p.equip-warn role="status" hidden { (icons::alert()) span { (tr.shop_unsupported) } }
                    }
                }
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
