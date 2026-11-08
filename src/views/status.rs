//! Page de statut : l'état des services dont Misty a besoin, vérifiés par le site lui-même.

use super::layout::{self, Head, Page};
use crate::status::{Etat, Groupe, Ligne, Role, SERVICES};
use maud::{Markup, html};

pub fn render(p: &Page, lignes: &[Ligne], age: Option<u64>) -> Markup {
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
        (Groupe::Misty, &tr.st_group_misty),
        (Groupe::Minecraft, &tr.st_group_minecraft),
        (Groupe::Mods, &tr.st_group_mods),
    ];
    let main = html! {
        section.hero.pn-hero {
            div.wrap { div.pn-copy {
                div.tags { span { (tr.st_label) } }
                h1 { (tr.st_h1_1) br; span.accent { (tr.st_h1_2) } }
                p.sub { (tr.st_sub) }
            } }
        }
        section.sec.status {
            div.wrap {
                div.st-banner.(classe) {
                    span.st-dot {}
                    strong { (titre) }
                    @if let Some(s) = age {
                        span.st-age { (tr.st_checked.replace("{0}", &s.to_string())) }
                    }
                }
                @for (groupe, nom) in groupes {
                    div.st-group {
                        h2 { (nom) }
                        div.st-list {
                            @for l in lignes.iter().filter(|l| SERVICES[l.index].groupe == groupe) {
                                (ligne(p, l))
                            }
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
            div.st-name {
                span.st-dot.(classe) {}
                div {
                    strong { (s.nom) }
                    span { (role) }
                }
            }
            div.st-bars title=(tr.st_history) aria-hidden="true" {
                @for e in &l.historique {
                    i.(e.map(Etat::slug).unwrap_or("none")) {}
                }
            }
            div.st-meta {
                span.st-state.(classe) { (etat) }
                span.st-num {
                    @if let Some(ms) = l.actuel.and_then(|m| m.latence) { (ms.as_millis()) " ms" }
                    @if let Some(d) = l.dispo { " · " (format!("{d:.1}")) " % " (tr.st_uptime) }
                }
            }
        }
    }
}
