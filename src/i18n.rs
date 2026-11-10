//! Traductions : un fichier TOML par langue dans `locales/`, intégré au binaire à la compilation.
//! Chaque clé est obligatoire : une clé manquante fait échouer `cargo test` (et le démarrage).

use axum::http::{HeaderMap, header};
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

/// Déclare `Tr` (un champ par clé des fichiers `locales/*.toml`) et sa lecture.
///
/// Pas de `#[derive(Deserialize)]` : avec plus de 300 champs, serde générait une fonction géante
/// (et en trois exemplaires) que LLVM mettait plus de dix minutes à optimiser en release. Ici, la
/// lecture est une suite d'affectations simples, avec les mêmes garanties : une clé manquante,
/// inconnue ou du mauvais type fait échouer le démarrage et `cargo test`.
macro_rules! traductions {
    ($($(#[$m:meta])* pub $champ:ident : $t:ty,)*) => {
        #[derive(Default)]
        pub struct Tr {
            $($(#[$m])* pub $champ: $t,)*
        }

        impl Tr {
            fn depuis(mut table: toml::Table, code: &str) -> Tr {
                let mut tr = Tr::default();
                $( tr.$champ = Lire::lire(table.remove(stringify!($champ)), code, stringify!($champ)); )*
                if let Some(cle) = table.keys().next() {
                    panic!("locales/{code}.toml : clé inconnue `{cle}`");
                }
                tr
            }
        }
    };
}

/// Lecture d'une valeur de traduction (texte, ou liste de 12 mois).
trait Lire: Sized {
    fn lire(valeur: Option<toml::Value>, code: &str, cle: &str) -> Self;
}

impl Lire for String {
    #[inline(never)]
    fn lire(valeur: Option<toml::Value>, code: &str, cle: &str) -> Self {
        match valeur {
            Some(toml::Value::String(s)) => s,
            Some(_) => panic!("locales/{code}.toml : `{cle}` doit être un texte"),
            None => panic!("locales/{code}.toml : clé `{cle}` manquante"),
        }
    }
}

impl Lire for [String; 12] {
    #[inline(never)]
    fn lire(valeur: Option<toml::Value>, code: &str, cle: &str) -> Self {
        let textes: Vec<String> = match valeur {
            Some(toml::Value::Array(a)) => a.into_iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
            None => panic!("locales/{code}.toml : clé `{cle}` manquante"),
            Some(_) => Vec::new(),
        };
        textes.try_into().unwrap_or_else(|_| panic!("locales/{code}.toml : `{cle}` doit contenir 12 textes"))
    }
}

traductions! {
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
    pub alt_versions: String,
    pub alt_skin: String,
    pub alt_horizon: String,
    pub hz_label: String,
    pub hz_title: String,
    pub hz_text: String,
    pub hz_p1: String,
    pub hz_p2: String,
    pub hz_p3: String,
    pub hz_site: String,
    pub hz_note: String,
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
    pub discord_join: String,
    pub discord_sub: String,
    pub ft_about_p: String,
    pub ft_disclaimer: String,
    pub ft_made: String,
    pub ft_top: String,
    pub ft_legal_t: String,
    pub legal_notice: String,
    pub legal_privacy: String,
    pub legal_terms: String,
    pub legal_updated: String,
    pub legal_untranslated: String,
    pub nav_tools: String,
    pub tools_title: String,
    pub tools_label: String,
    pub tools_h1_1: String,
    pub tools_h1_2: String,
    pub tools_sub: String,
    pub tools_soon: String,
    pub tools_open: String,
    pub tool_seed_t: String,
    pub tool_seed_p: String,
    pub tool_nether_t: String,
    pub tool_nether_p: String,
    pub tool_slime_t: String,
    pub tool_slime_p: String,
    pub tool_cmd_t: String,
    pub tool_cmd_p: String,
    pub seed_title: String,
    pub seed_h1: String,
    pub seed_sub: String,
    pub seed_placeholder: String,
    pub seed_go: String,
    pub seed_random: String,
    pub seed_version: String,
    pub seed_overworld: String,
    pub seed_nether: String,
    pub seed_end: String,
    pub seed_structures: String,
    pub seed_spawn: String,
    pub seed_origin: String,
    pub seed_zoom_in: String,
    pub seed_zoom_out: String,
    pub seed_share: String,
    pub seed_copied: String,
    pub seed_help: String,
    pub seed_zoom_more: String,
    pub seed_error: String,
    pub seed_powered: String,
    pub sv_village: String,
    pub sv_outpost: String,
    pub sv_mansion: String,
    pub sv_monument: String,
    pub sv_temple: String,
    pub sv_ruined_portal: String,
    pub sv_ancient_city: String,
    pub sv_trial_chambers: String,
    pub sv_fortress: String,
    pub sv_bastion: String,
    pub sv_end_city: String,
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
    pub nav_status: String,
    pub st_title: String,
    pub st_label: String,
    pub st_h1_1: String,
    pub st_h1_2: String,
    pub st_sub: String,
    pub st_all_ok: String,
    pub st_some_slow: String,
    pub st_some_down: String,
    pub st_waiting: String,
    pub st_waiting_short: String,
    pub st_checked: String,
    pub st_up: String,
    pub st_slow: String,
    pub st_down: String,
    pub st_uptime_30: String,
    pub st_hour_ago: String,
    pub st_now: String,
    pub st_no_data: String,
    pub st_group_misty_desc: String,
    pub st_group_minecraft_desc: String,
    pub st_group_mods_desc: String,
    pub st_role_horizon: String,
    pub st_incidents: String,
    pub st_no_incidents: String,
    pub st_inc_down: String,
    pub st_inc_slow: String,
    pub st_resolved: String,
    pub st_ongoing: String,
    pub vf_title: String,
    pub vf_sub: String,
    pub vf_input: String,
    pub vf_submit: String,
    pub vf_new: String,
    pub vf_hint: String,
    pub vf_wrong: String,
    pub vf_ok_title: String,
    pub vf_ok_p: String,
    pub vf_bad_title: String,
    pub vf_bad_p: String,
    pub vf_off_title: String,
    pub vf_off_p: String,
    pub vf_many_p: String,
    pub vf_role_title: String,
    pub vf_role_p: String,
    pub vf_loading: String,
    pub vf_close: String,
    pub vf_auth_p: String,
    pub vf_auth_t: String,
    pub vf_autoclose: String,
    pub vf_title_short: String,
    pub vf_for: String,
    pub vf_step_1: String,
    pub vf_step_2: String,
    pub vf_step_3: String,
    pub vf_consent_p: String,
    pub vf_drag: String,
    pub vf_slide: String,
    pub vf_here_t: String,
    pub vf_here_p: String,
    pub vf_already_t: String,
    pub vf_already_p: String,
    pub st_group_misty: String,
    pub st_group_minecraft: String,
    pub st_group_mods: String,
    pub st_role_download: String,
    pub st_role_notes: String,
    pub st_role_accounts: String,
    pub st_role_login: String,
    pub st_role_skins: String,
    pub st_role_versions: String,
    pub st_role_assets: String,
    pub st_role_libraries: String,
    pub st_role_mods: String,
    pub st_role_loaders: String,
    pub st_how: String,
}

fn parse(code: &str, src: &str) -> Tr {
    let table: toml::Table =
        toml::from_str(src).unwrap_or_else(|e| panic!("locales/{code}.toml invalide : {e}"));
    let tr = Tr::depuis(table, code);
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

    /// Une clé manquante, en trop ou du mauvais type est refusée (comme avec serde avant).
    #[test]
    fn locale_incomplete_refusee() {
        let fr = include_str!("../locales/fr.toml");
        let sans_titre: String = fr.lines().filter(|l| !l.starts_with("title =")).collect::<Vec<_>>().join("\n");
        let en_trop = format!("{fr}\nzz_inconnue = \"x\"\n");
        let mauvais_type = fr.replace("title = ", "title = 3 #");
        for (cas, attendu) in [(sans_titre, "clé `title` manquante"), (en_trop, "clé inconnue `zz_inconnue`"), (mauvais_type, "`title` doit être un texte")] {
            let erreur = std::panic::catch_unwind(|| parse("fr", &cas)).err().expect(attendu);
            let message = erreur.downcast_ref::<String>().cloned().unwrap_or_default();
            assert!(message.contains(attendu), "{message}");
        }
    }

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
