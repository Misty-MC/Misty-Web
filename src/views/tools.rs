//! Outils : la page d'accueil des outils (`/tools`) et le seed viewer (`/tools/seed`).

use super::icons;
use super::layout::{self, Head, Page};
use crate::seedmap;
use maud::{Markup, PreEscaped, html};
use serde_json::json;

/// Un outil de la page `/tools` : (icône, titre, texte, lien, ou `None` s'il arrive bientôt).
fn outils<'a>(p: &Page<'a>) -> Vec<(Markup, &'static str, &'static str, Option<&'static str>)> {
    let tr = p.tr;
    vec![
        (icons::globe(), &tr.tool_seed_t, &tr.tool_seed_p, Some("/tools/seed")),
        (icons::layers(), &tr.tool_nether_t, &tr.tool_nether_p, None),
        (icons::grid(), &tr.tool_slime_t, &tr.tool_slime_p, None),
        (icons::notes(), &tr.tool_cmd_t, &tr.tool_cmd_p, None),
    ]
}

pub fn index(p: &Page) -> Markup {
    let tr = p.tr;
    let main = html! {
        section.tools-hero {
            div.wrap {
                span.label { (tr.tools_label) }
                h1 { (tr.tools_h1_1) " " span.accent { (tr.tools_h1_2) } }
                p.sub { (tr.tools_sub) }
            }
        }
        section.wrap.tools-grid {
            @for (icone, titre, texte, lien) in outils(p) {
                @if let Some(href) = lien {
                    a.tool.reveal href=(href) {
                        span.ico { (icone) }
                        h2 { (titre) }
                        p { (texte) }
                        span.go { (tr.tools_open) " →" }
                    }
                } @else {
                    div.tool.soon.reveal {
                        span.ico { (icone) }
                        h2 { (titre) span.badge { (tr.tools_soon) } }
                        p { (texte) }
                    }
                }
            }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.tools_title,
            description: &tr.tools_sub,
            jsonld: None,
            css: &["tools"],
            scripts: &[],
            body_class: "tools",
        },
        main,
    )
}

pub fn seed(p: &Page) -> Markup {
    let tr = p.tr;
    let structures: Vec<(&str, &str)> = {
        let mut vus = Vec::new();
        for (cle, _, _) in seedmap::STRUCTURES {
            if !vus.iter().any(|(c, _)| c == cle) {
                vus.push((*cle, structure_label(tr, cle)));
            }
        }
        vus
    };
    // Textes et réglages lus par seed-viewer.js
    let config = json!({
        "versions": seedmap::VERSIONS.iter().map(|(nom, v)| json!([nom, v])).collect::<Vec<_>>(),
        "scales": seedmap::ECHELLES,
        "tile": seedmap::TUILE,
        "region": seedmap::REGION,
        "structures": seedmap::STRUCTURES.iter().map(|(cle, dim, types)| json!([cle, dim, types])).collect::<Vec<_>>(),
        // calcul dans le navigateur : Web Workers + cubiomes en WebAssembly
        "worker": format!("/js/seed-worker.js?v={}", layout::ASSET_VERSION.as_str()),
        "wasm": format!("/wasm/cubiomes.wasm?v={}", layout::ASSET_VERSION.as_str()),
        "labels": structures.iter().map(|(c, l)| (c.to_string(), json!(l))).collect::<serde_json::Map<_, _>>(),
        "text": {
            "copied": tr.seed_copied,
            "zoomMore": tr.seed_zoom_more,
            "error": tr.seed_error,
            "spawn": tr.seed_spawn,
        },
    });
    let main = html! {
        section.wrap.seed-head {
            nav.crumbs aria-label="Breadcrumb" {
                a href="/tools" { (tr.tools_label) }
                span { "/" }
                span { (tr.seed_h1) }
            }
            h1 { (tr.seed_h1) }
            p.sub { (tr.seed_sub) }
        }
        section.wrap.seed-app #seed-app {
            form.seed-bar #seed-form {
                label.seed-input {
                    span.sr { (tr.seed_placeholder) }
                    input #seed-text type="text" name="seed" maxlength="64" placeholder=(tr.seed_placeholder) autocomplete="off" spellcheck="false";
                }
                button.btn.play.sm type="submit" { (tr.seed_go) }
                button.btn.ghost.sm #seed-random type="button" { (icons::refresh()) span { (tr.seed_random) } }
                label.seed-select {
                    span { (tr.seed_version) }
                    select #seed-version {
                        @for (nom, v) in seedmap::VERSIONS { option value=(v) { (nom) } }
                    }
                }
            }
            div.seed-options {
                div.seed-dims role="radiogroup" aria-label="Dimension" {
                    button type="button" data-dim="0" aria-pressed="true" { (tr.seed_overworld) }
                    button type="button" data-dim="-1" aria-pressed="false" { (tr.seed_nether) }
                    button type="button" data-dim="1" aria-pressed="false" { (tr.seed_end) }
                }
                div.seed-structs #seed-structs aria-label=(tr.seed_structures) {
                    @for (cle, label) in &structures {
                        label.chip data-key=(cle) {
                            // les portails en ruine sont partout : cachés au départ
                            input type="checkbox" checked[*cle != "ruined_portal"] value=(cle);
                            i.sw data-key=(cle) {}
                            (label)
                        }
                    }
                }
            }
            div.seed-map #seed-map {
                canvas #seed-canvas {}
                div.seed-tools {
                    button type="button" #seed-zoom-in aria-label=(tr.seed_zoom_in) title=(tr.seed_zoom_in) { "+" }
                    button type="button" #seed-zoom-out aria-label=(tr.seed_zoom_out) title=(tr.seed_zoom_out) { "−" }
                    button type="button" #seed-go-spawn { (tr.seed_spawn) }
                    button type="button" #seed-go-origin { (tr.seed_origin) }
                    button type="button" #seed-share { (tr.seed_share) }
                }
                p.seed-note #seed-note hidden {}
                div.seed-status {
                    span #seed-biome { "–" }
                    span #seed-coords { "X 0 · Z 0" }
                }
            }
            p.seed-foot { (tr.seed_help) " " (tr.seed_powered) }
            script #seed-config type="application/json" { (PreEscaped(config.to_string().replace("</", "<\\/"))) }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.seed_title,
            description: &tr.seed_sub,
            jsonld: None,
            css: &["tools"],
            scripts: &["/js/seed-viewer.js"],
            body_class: "tools seed",
        },
        main,
    )
}

fn structure_label<'a>(tr: &'a crate::i18n::Tr, cle: &str) -> &'a str {
    match cle {
        "village" => &tr.sv_village,
        "outpost" => &tr.sv_outpost,
        "mansion" => &tr.sv_mansion,
        "monument" => &tr.sv_monument,
        "temple" => &tr.sv_temple,
        "ruined_portal" => &tr.sv_ruined_portal,
        "ancient_city" => &tr.sv_ancient_city,
        "trial_chambers" => &tr.sv_trial_chambers,
        "fortress" => &tr.sv_fortress,
        "bastion" => &tr.sv_bastion,
        _ => &tr.sv_end_city,
    }
}
