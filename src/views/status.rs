//! Page de statut : l'état des services dont Misty a besoin, vérifiés par le site lui-même.
//! Une case = un jour (30 jours), disponibilité globale, et les incidents des 90 derniers jours.

use super::layout::{self, Head, Page};
use crate::format;
use crate::status::{Etat, Groupe, Incident, Ligne, Role, SERVICES};
use maud::{Markup, html};

/// « 99,98 % » selon la langue.
fn pourcent(p: &Page, v: f32) -> String {
    format!("{v:.2}").replace('.', &p.tr.decimal_sep) + " %"
}

pub fn render(p: &Page, lignes: &[Ligne], incidents: &[Incident], dispo: Option<f32>, age: Option<u64>) -> Markup {
    let tr = p.tr;
    let mesurees: Vec<&Ligne> = lignes.iter().filter(|l| l.actuel.is_some()).collect();
    let hors_ligne = mesurees.iter().filter(|l| l.actuel.map(|m| m.etat) == Some(Etat::HorsLigne)).count();
    let lents = mesurees.iter().filter(|l| l.actuel.map(|m| m.etat) == Some(Etat::Lent)).count();
    let (classe, titre) = if mesurees.is_empty() {
        ("wait", &tr.st_waiting)
    } else if hors_ligne > 0 {
        ("down", &tr.st_some_down)
    } else if lents > 0 {
        ("slow", &tr.st_some_slow)
    } else {
        ("up", &tr.st_all_ok)
    };
    let groupes = [
        (Groupe::Misty, &tr.st_group_misty, &tr.st_group_misty_desc),
        (Groupe::Minecraft, &tr.st_group_minecraft, &tr.st_group_minecraft_desc),
        (Groupe::Mods, &tr.st_group_mods, &tr.st_group_mods_desc),
    ];
    let main = html! {
        section.hero.pn-hero.st-hero {
            div.wrap { div.pn-copy {
                div.tags { span { (tr.st_label) } }
                h1 { (tr.st_h1_1) " " span.accent { (tr.st_h1_2) } }
                p.sub { (tr.st_sub) }
                div.st-summary {
                    div.st-banner.(classe) {
                        span.st-dot {}
                        div {
                            strong { (titre) }
                            @if let Some(s) = age {
                                span.st-age { (tr.st_checked.replace("{0}", &s.to_string())) }
                            }
                        }
                    }
                    div.st-kpi {
                        strong { @if let Some(d) = dispo { (pourcent(p, d)) } @else { "—" } }
                        span { (tr.st_uptime_30) }
                    }
                }
            } }
        }
        section.sec.status {
            div.wrap {
                @for (groupe, nom, desc) in groupes {
                    div.st-group {
                        div.st-ghead {
                            h2 { (nom) }
                            p { (desc) }
                        }
                        div.st-list {
                            @for l in lignes.iter().filter(|l| SERVICES[l.index].groupe == groupe) {
                                (ligne(p, l))
                            }
                        }
                    }
                }
                div.st-group.st-incidents {
                    div.st-ghead { h2 { (tr.st_incidents) } }
                    @if incidents.is_empty() {
                        p.st-none { span.st-dot.up {} (tr.st_no_incidents) }
                    } @else {
                        div.st-inc-list {
                            @for i in incidents { (incident(p, i)) }
                        }
                    }
                }
                p.st-how { (tr.st_how) " " a href="/status.json" { "status.json" } }
            }
        }
    };
    layout::render(
        p,
        Head {
            title: &tr.st_title,
            description: &tr.st_sub,
            jsonld: None,
            css: &["home", "patch-notes", "status"],
            scripts: &[],
            body_class: "status-page",
        },
        main,
    )
}

fn ligne(p: &Page, l: &Ligne) -> Markup {
    let tr = p.tr;
    let s = &SERVICES[l.index];
    let role = match s.role {
        Role::Telechargement => &tr.st_role_download,
        Role::Notes => &tr.st_role_notes,
        Role::Horizon => &tr.st_role_horizon,
        Role::Comptes => &tr.st_role_accounts,
        Role::Connexion => &tr.st_role_login,
        Role::Skins => &tr.st_role_skins,
        Role::Versions => &tr.st_role_versions,
        Role::Ressources => &tr.st_role_assets,
        Role::Bibliotheques => &tr.st_role_libraries,
        Role::Mods => &tr.st_role_mods,
        Role::Loaders => &tr.st_role_loaders,
    };
    let (classe, etat) = match l.actuel.map(|m| m.etat) {
        Some(Etat::EnLigne) => ("up", &tr.st_up),
        Some(Etat::Lent) => ("slow", &tr.st_slow),
        Some(Etat::HorsLigne) => ("down", &tr.st_down),
        None => ("wait", &tr.st_waiting_short),
    };
    html! {
        div.st-row {
            div.st-top {
                div.st-name {
                    strong { (s.nom) }
                    span { (role) }
                }
                div.st-right {
                    span.st-state.(classe) { span.st-dot.(classe) {} (etat) }
                    @if let Some(d) = l.dispo { span.st-num { (pourcent(p, d)) } }
                }
            }
            div.st-bars {
                @for (date, jour) in &l.historique {
                    @let quand = format::jour(tr, *date, true);
                    @match jour {
                        Some(j) => {
                            i.(j.etat().slug()) title=(format!("{quand} · {} · {}", pourcent(p, j.dispo()), tr.st_checks.replace("{0}", &j.total().to_string()))) {}
                        }
                        None => { i.none title=(format!("{quand} · {}", tr.st_no_data)) {} }
                    }
                }
            }
            div.st-legend {
                span { (tr.st_days_ago) }
                span.ms { @if let Some(ms) = l.actuel.and_then(|m| m.latence) { (ms.as_millis()) " ms" } }
                span { (tr.st_today) }
            }
        }
    }
}

fn incident(p: &Page, i: &Incident) -> Markup {
    let tr = p.tr;
    let (classe, titre) = if i.panne { ("down", &tr.st_inc_down) } else { ("slow", &tr.st_inc_slow) };
    html! {
        article.st-inc {
            div.st-inc-h {
                span.st-dot.(classe) {}
                h3 { (titre) " · " (i.service) }
                span.st-tag.(if i.fin.is_some() { "done" } else { "open" }) {
                    @if i.fin.is_some() { (tr.st_resolved) } @else { (tr.st_ongoing) }
                }
            }
            p {
                (format::moment(tr, i.debut))
                @if let Some(f) = i.fin {
                    " — " (format::moment(tr, f))
                    " · " (duree(f - i.debut))
                }
            }
        }
    }
}

/// « 3 min », « 2 h 05 ».
fn duree(secondes: i64) -> String {
    let min = (secondes.max(60) + 30) / 60;
    if min < 60 { format!("{min} min") } else { format!("{} h {:02}", min / 60, min % 60) }
}
