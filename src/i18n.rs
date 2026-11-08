//! Traductions : un fichier TOML par langue dans `locales/`, intégré au binaire à la compilation.
//! Chaque clé est obligatoire : une clé manquante fait échouer `cargo test` (et le démarrage).

use axum::http::{HeaderMap, header};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Fr,
    En,
    Es,
    Ru,
    It,
}

impl Lang {
    pub const ALL: [Lang; 5] = [Lang::Fr, Lang::En, Lang::Es, Lang::Ru, Lang::It];

    pub fn code(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
            Lang::Es => "es",
            Lang::Ru => "ru",
            Lang::It => "it",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        let code = code.get(..2)?.to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }

    /// L'application n'existe qu'en FR et EN : les autres langues utilisent les captures anglaises.
    pub fn screens_dir(self) -> &'static str {
        if self == Lang::Fr { "fr" } else { "en" }
    }

    pub fn tr(self) -> &'static Tr {
        &TRANSLATIONS[Lang::ALL.iter().position(|l| *l == self).unwrap()]
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Tr {
    pub code: String,
    pub lang_name: String,
    pub title: String,
    pub description: String,
    pub date_order: String,
    pub months: [String; 12],
    pub decimal_sep: String,
    pub mb: String,
    pub og_locale: String,
    pub nav_features: String,
    pub nav_loaders: String,
    pub nav_faq: String,
    pub nav_patch_notes: String,
    pub nav_download: String,
    pub nav_language: String,
    pub hero_tag_mc: String,
    pub hero_tag_rust: String,
    pub hero_title_1: String,
    pub hero_title_2: String,
    pub hero_sub: String,
    pub hero_cta_download: String,
    pub hero_cta_notes: String,
    pub hero_free: String,
    pub card_label: String,
    pub card_whats_new: String,
    pub card_more: String,
    pub card_unavailable: String,
    pub card_no_notes: String,
    pub prev_label: String,
    pub prev_title: String,
    pub prev_text: String,
    pub alt_home: String,
    pub alt_profiles: String,
    pub alt_textures: String,
    pub f1_t: String,
    pub f1_p: String,
    pub f2_t: String,
    pub f2_p: String,
    pub f3_t: String,
    pub f3_p: String,
    pub experimental: String,
    pub feat_label: String,
    pub feat_title: String,
    pub feat_text: String,
    pub c1_t: String,
    pub c1_p: String,
    pub c2_t: String,
    pub c2_p: String,
    pub c3_t: String,
    pub c3_p: String,
    pub c4_t: String,
    pub c4_p: String,
    pub c5_t: String,
    pub c5_p: String,
    pub c6_t: String,
    pub c6_p: String,
    pub load_label: String,
    pub load_caption: String,
    pub load_title: String,
    pub load_text: String,
    pub load_item: String,
    pub load_sources: String,
    pub dl_label: String,
    pub dl_title: String,
    pub dl_text: String,
    pub recommended: String,
    pub inst_t: String,
    pub inst_p: String,
    pub port_t: String,
    pub port_p: String,
    pub notes_t: String,
    pub notes_p: String,
    pub github_t: String,
    pub github_p: String,
    pub faq_label: String,
    pub faq_title: String,
    pub q1: String,
    pub a1: String,
    pub q2: String,
    pub a2: String,
    pub q3: String,
    pub a3: String,
    pub q4: String,
    pub a4: String,
    pub q5: String,
    pub a5: String,
    pub cta_title: String,
    pub cta_text: String,
    pub ft_tag: String,
    pub ft_nav: String,
    pub ft_about_t: String,
    pub ft_about_p: String,
    pub ft_disclaimer: String,
    pub ft_made: String,
    pub ft_top: String,
    pub pn_title: String,
    pub pn_label: String,
    pub pn_h1_1: String,
    pub pn_h1_2: String,
    pub pn_sub: String,
    pub pn_browse: String,
    pub pn_all_github: String,
    pub pn_filter_all: String,
    pub pn_filter_stable: String,
    pub pn_filter_pre: String,
    pub pn_badge_latest: String,
    pub pn_badge_stable: String,
    pub pn_badge_pre: String,
    pub pn_empty: String,
    pub pn_files: String,
    pub pn_view_github: String,
    pub pn_unavailable: String,
    pub nf_title: String,
    pub nf_text: String,
    pub nf_back: String,
    pub nav_docs: String,
    pub docs_title: String,
    pub docs_label: String,
    pub docs_h1_1: String,
    pub docs_h1_2: String,
    pub docs_sub: String,
    pub docs_search: String,
    pub docs_no_results: String,
    pub docs_on_page: String,
    pub docs_prev: String,
    pub docs_next: String,
    pub docs_edit: String,
    pub docs_report: String,
    pub docs_untranslated: String,
    pub docs_menu: String,
    pub docs_sec_start: String,
    pub docs_sec_play: String,
    pub docs_sec_tools: String,
    pub docs_sec_help: String,
    pub docs_sec_dev: String,
    pub nav_shop: String,
    pub shop_title: String,
    pub shop_label: String,
    pub shop_h1_1: String,
    pub shop_h1_2: String,
    pub shop_sub: String,
    pub shop_username: String,
    pub shop_placeholder: String,
    pub shop_continue: String,
    pub shop_invalid: String,
    pub shop_hint: String,
    pub shop_soon: String,
    pub shop_change: String,
    pub shop_badge: String,
    pub shop_cat_capes: String,
    pub shop_cat_capes_p: String,
    pub shop_cat_hats: String,
    pub shop_cat_hats_p: String,
    pub shop_cat_wings: String,
    pub shop_cat_wings_p: String,
    pub shop_cat_effects: String,
    pub shop_cat_effects_p: String,
    pub shop_preview: String,
    pub shop_drag: String,
    pub shop_view: String,
    pub shop_view_front: String,
    pub shop_view_side: String,
    pub shop_view_back: String,
    pub shop_anim: String,
    pub shop_anim_idle: String,
    pub shop_anim_walk: String,
    pub shop_anim_run: String,
    pub shop_anim_fly: String,
    pub shop_anim_wave: String,
    pub shop_anim_crouch: String,
    pub shop_own_cape: String,
    pub shop_elytra: String,
    pub shop_rotate: String,
    pub shop_reload: String,
    pub shop_reload_hint: String,
    pub shop_model_classic: String,
    pub shop_model_slim: String,
    pub shop_copy_uuid: String,
    pub shop_copied: String,
    pub shop_namemc: String,
    pub shop_cosmetics_t: String,
    pub shop_cosmetics_p: String,
    pub shop_not_found: String,
    pub shop_no_cape: String,
    pub shop_item_cape: String,
    pub shop_item_elytra: String,
    pub shop_equip: String,
    pub shop_unequip: String,
    pub shop_unsupported: String,
    pub shop_cat_bodywear: String,
    pub shop_cat_bodywear_p: String,
    pub shop_cat_pets: String,
    pub shop_cat_pets_p: String,
    pub shop_cat_emotes: String,
    pub shop_cat_emotes_p: String,
    pub shop_cat_bundles: String,
    pub shop_cat_bundles_p: String,
}

fn parse(code: &str, src: &str) -> Tr {
    let tr: Tr =
        toml::from_str(src).unwrap_or_else(|e| panic!("locales/{code}.toml invalide : {e}"));
    assert_eq!(tr.code, code, "locales/{code}.toml : `code` incorrect");
    tr
}

static TRANSLATIONS: LazyLock<[Tr; 5]> = LazyLock::new(|| {
    [
        parse("fr", include_str!("../locales/fr.toml")),
        parse("en", include_str!("../locales/en.toml")),
        parse("es", include_str!("../locales/es.toml")),
        parse("ru", include_str!("../locales/ru.toml")),
        parse("it", include_str!("../locales/it.toml")),
    ]
});

/// Force le chargement des traductions (appelé au démarrage pour échouer tôt).
pub fn init() {
    LazyLock::force(&TRANSLATIONS);
}

pub const COOKIE: &str = "misty-lang";

/// Choisit la langue : paramètre `?lang=`, puis cookie, puis en-tête `Accept-Language`, sinon anglais.
/// Renvoie aussi `true` si la langue vient du paramètre (il faut alors poser le cookie).
pub fn negotiate(query: Option<&str>, headers: &HeaderMap) -> (Lang, bool) {
    if let Some(l) = query.and_then(Lang::from_code) {
        return (l, true);
    }
    let cookie = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|c| c.trim().split_once('='))
        .find(|(k, _)| *k == COOKIE)
        .and_then(|(_, v)| Lang::from_code(v));
    if let Some(l) = cookie {
        return (l, false);
    }
    let accept = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let from_header = accept
        .split(',')
        .filter_map(|part| Lang::from_code(part.trim()))
        .next();
    (from_header.unwrap_or(Lang::En), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_locales_load() {
        for l in Lang::ALL {
            assert_eq!(l.tr().code, l.code());
        }
    }

    #[test]
    fn negotiation_order() {
        let mut h = HeaderMap::new();
        h.insert(
            header::ACCEPT_LANGUAGE,
            "es-ES,es;q=0.9,en;q=0.8".parse().unwrap(),
        );
        assert_eq!(negotiate(None, &h), (Lang::Es, false));
        h.insert(header::COOKIE, "a=b; misty-lang=ru".parse().unwrap());
        assert_eq!(negotiate(None, &h), (Lang::Ru, false));
        assert_eq!(negotiate(Some("it"), &h), (Lang::It, true));
        assert_eq!(negotiate(Some("xx"), &HeaderMap::new()), (Lang::En, false));
    }
}
