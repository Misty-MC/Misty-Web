//! Routes HTTP : pages, API JSON et téléchargements.

use crate::AppState;
use crate::docs;
use crate::github::{self, Kind};
use crate::i18n::{self, Lang};
use crate::skins;
use crate::views::docs as docs_view;
use crate::views::shop::{self, ShopView};
use crate::views::status as status_view;
use crate::views::verify as verify_view;
use crate::views::{home, layout::Page, not_found, patch_notes};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Json, Response};
use maud::Markup;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Default)]
pub struct PageQuery {
    lang: Option<String>,
    filter: Option<String>,
    /// Boutique : pseudo Minecraft saisi.
    player: Option<String>,
    /// Boutique : revenir au formulaire pour changer de pseudo.
    change: Option<String>,
}

/// Réponse HTML : pose le cookie de langue si elle vient d'être choisie avec `?lang=`.
fn html_response(status: StatusCode, markup: Markup, lang: Lang, remember: bool) -> Response {
    let mut res = (status, markup).into_response();
    let h = res.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    h.insert(
        header::VARY,
        HeaderValue::from_static("Cookie, Accept-Language"),
    );
    if remember {
        let cookie = format!(
            "{}={}; Path=/; Max-Age=31536000; SameSite=Lax",
            i18n::COOKIE,
            lang.code()
        );
        if let Ok(v) = HeaderValue::from_str(&cookie) {
            h.insert(header::SET_COOKIE, v);
        }
    }
    res
}

fn page<'a>(state: &'a AppState, lang: Lang, path: &'a str, count: Option<usize>) -> Page<'a> {
    Page {
        lang,
        tr: lang.tr(),
        path,
        release_count: count,
        repo: state.github.repo(),
        fog: &state.fog_strip,
        scene: &state.scene,
        site_url: &state.site_url,
    }
}

pub async fn index(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
    headers: HeaderMap,
) -> Response {
    let (lang, remember) = i18n::negotiate(q.lang.as_deref(), &headers);
    let releases = state.github.releases().await;
    let latest = releases.as_deref().and_then(|l| github::latest(l));
    let p = page(&state, lang, "/", releases.as_ref().map(|l| l.len()));
    html_response(StatusCode::OK, home::render(&p, latest), lang, remember)
}

pub async fn patch_notes(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
    headers: HeaderMap,
) -> Response {
    let (lang, remember) = i18n::negotiate(q.lang.as_deref(), &headers);
    let releases = state.github.releases().await;
    let p = page(
        &state,
        lang,
        "/patch-notes",
        releases.as_ref().map(|l| l.len()),
    );
    let filter = patch_notes::Filter::from_query(q.filter.as_deref());
    let markup = patch_notes::render(&p, releases.as_deref().map(|v| v.as_slice()), filter);
    html_response(StatusCode::OK, markup, lang, remember)
}

pub async fn docs_index(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
    headers: HeaderMap,
) -> Response {
    let (lang, remember) = i18n::negotiate(q.lang.as_deref(), &headers);
    let count = state.github.releases().await.map(|l| l.len());
    let p = page(&state, lang, "/docs", count);
    html_response(
        StatusCode::OK,
        docs_view::index(&p, docs::get(lang)),
        lang,
        remember,
    )
}

pub async fn docs_page(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Query(q): Query<PageQuery>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let (lang, remember) = i18n::negotiate(q.lang.as_deref(), &headers);
    let set = docs::get(lang);
    let Some(idx) = set.find(&slug) else {
        return not_found(State(state), uri, headers).await;
    };
    let count = state.github.releases().await.map(|l| l.len());
    let path = uri.path().to_string();
    let p = page(&state, lang, &path, count);
    html_response(
        StatusCode::OK,
        docs_view::page(&p, set, idx),
        lang,
        remember,
    )
}

const PLAYER_COOKIE: &str = "misty-player";

fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|c| c.trim().split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
}

/// Boutique : formulaire du pseudo, puis « bientôt disponible ».
/// Le pseudo validé est retenu dans un cookie pour les visites suivantes.
pub async fn shop(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
    headers: HeaderMap,
) -> Response {
    let (lang, remember) = i18n::negotiate(q.lang.as_deref(), &headers);
    let count = state.github.releases().await.map(|l| l.len());
    let p = page(&state, lang, "/shop", count);
    let saved = read_cookie(&headers, PLAYER_COOKIE).filter(|n| shop::valid_username(n));
    let typed = q.player.as_deref().map(str::trim);

    let (view, new_player) = match (typed, q.change.is_some(), saved) {
        (Some(name), _, _) if shop::valid_username(name) => {
            (ShopView::Studio { player: name }, Some(name))
        }
        (Some(name), _, _) => (
            ShopView::Form {
                value: name,
                invalid: !name.is_empty(),
            },
            None,
        ),
        (None, false, Some(name)) => (ShopView::Studio { player: name }, None),
        (None, _, saved) => (
            ShopView::Form {
                value: saved.unwrap_or(""),
                invalid: false,
            },
            None,
        ),
    };
    let mut res = html_response(StatusCode::OK, shop::render(&p, view), lang, remember);
    if let Some(name) = new_player {
        let cookie = format!("{PLAYER_COOKIE}={name}; Path=/; Max-Age=31536000; SameSite=Lax");
        if let Ok(v) = HeaderValue::from_str(&cookie) {
            res.headers_mut().append(header::SET_COOKIE, v);
        }
    }
    res
}

fn texture_response(png: axum::body::Bytes) -> Response {
    let mut res = Body::from(png).into_response();
    let h = res.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=600"),
    );
    res
}

async fn lookup_player(
    state: &AppState,
    name: &str,
    fresh: bool,
) -> Result<Arc<skins::Profile>, Response> {
    let name = name.strip_suffix(".png").unwrap_or(name);
    if !shop::valid_username(name) {
        return Err((StatusCode::NOT_FOUND, "Pseudo invalide").into_response());
    }
    match state.skins.lookup(state.github.http(), name, fresh).await {
        skins::Lookup::Found(p) => Ok(p),
        skins::Lookup::NotFound => {
            Err((StatusCode::NOT_FOUND, "Joueur introuvable").into_response())
        }
        skins::Lookup::Unavailable => {
            Err((StatusCode::BAD_GATEWAY, "Skin indisponible").into_response())
        }
    }
}

/// `/skin/<pseudo>` (ou `/skin/<pseudo>.png`) : le skin du joueur, relayé depuis le site
/// pour que l'aperçu 3D puisse le lire.
pub async fn skin(State(state): State<Arc<AppState>>, Path(name): Path<String>) -> Response {
    match lookup_player(&state, &name, false).await {
        Ok(p) => texture_response(p.skin.clone()),
        Err(res) => res,
    }
}

/// `/cape/<pseudo>` : la cape Mojang du joueur, s'il en a une.
pub async fn cape(State(state): State<Arc<AppState>>, Path(name): Path<String>) -> Response {
    match lookup_player(&state, &name, false).await {
        Ok(p) => match &p.cape {
            Some(cape) => texture_response(cape.clone()),
            None => (StatusCode::NOT_FOUND, "Pas de cape").into_response(),
        },
        Err(res) => res,
    }
}

#[derive(Deserialize)]
pub struct PlayerQuery {
    /// `?fresh=1` : recharge le profil chez Mojang au lieu d'utiliser le cache.
    fresh: Option<String>,
}

/// `/player/<pseudo>` : pseudo exact, UUID, modèle de bras et présence d'une cape (JSON).
pub async fn player(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(q): Query<PlayerQuery>,
) -> Response {
    let fresh = q.fresh.is_some();
    match lookup_player(&state, &name, fresh).await {
        Ok(p) => {
            let mut res = Json(serde_json::json!({
                "name": p.name,
                "uuid": p.uuid.as_deref().map(skins::dashed_uuid),
                "slim": p.slim,
                "cape": p.cape.is_some(),
            }))
            .into_response();
            res.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=600"),
            );
            res
        }
        Err(res) => res,
    }
}

pub async fn robots(State(state): State<Arc<AppState>>) -> Response {
    let mut res = crate::seo::robots_txt(&state.site_url).into_response();
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    res
}

pub async fn sitemap(State(state): State<Arc<AppState>>) -> Response {
    let mut res = crate::seo::sitemap_xml(&state.site_url).into_response();
    let h = res.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/xml; charset=utf-8"),
    );
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    res
}

/// Index de recherche plein texte de la doc, chargé par site.js à la première recherche.
pub async fn docs_search_index(Query(q): Query<PageQuery>, headers: HeaderMap) -> Response {
    let (lang, _) = i18n::negotiate(q.lang.as_deref(), &headers);
    let entries: Vec<_> = docs::get(lang)
        .pages
        .iter()
        .map(|p| serde_json::json!({ "slug": p.slug, "text": p.text }))
        .collect();
    let mut res = Json(entries).into_response();
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=300"),
    );
    res
}

pub async fn not_found(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let (lang, _) = i18n::negotiate(None, &headers);
    let path = uri.path().to_string();
    let p = page(&state, lang, &path, None);
    html_response(StatusCode::NOT_FOUND, not_found::render(&p), lang, false)
}

/// Infos de la dernière version (même format que l'ancienne fonction Cloudflare).
pub async fn api_latest(State(state): State<Arc<AppState>>) -> Response {
    let releases = state.github.releases().await;
    let Some(r) = releases.as_deref().and_then(|l| github::latest(l)) else {
        let mut res = (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "unavailable" })),
        )
            .into_response();
        res.headers_mut().insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
        return res;
    };
    let mut res = Json(serde_json::json!({
        "version": r.tag_name,
        "date": r.published_at,
        "notes": r.notes(),
        "setupSize": r.asset(Kind::Setup).map(|a| a.size),
        "portableSize": r.asset(Kind::Portable).map(|a| a.size),
    }))
    .into_response();
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=120"),
    );
    res
}

/// `/download/setup` et `/download/portable` : le fichier de la dernière version, dans l'ordre
/// 1. la copie sur le serveur (voir `miroir`) ;
/// 2. sinon, le fichier relayé depuis GitHub (l'utilisateur reste sur le domaine du site) ;
/// 3. sinon, une redirection vers GitHub : un téléchargement ne finit jamais sur une erreur.
pub async fn download(
    State(state): State<Arc<AppState>>,
    Path(kind): Path<String>,
    req: axum::extract::Request,
) -> Response {
    let Some(kind) = Kind::from_slug(&kind) else {
        return (StatusCode::NOT_FOUND, "Type inconnu").into_response();
    };
    let page_github = format!("https://github.com/{}/releases/latest", state.github.repo());
    let releases = state.github.releases().await;
    let Some(asset) = releases
        .as_deref()
        .and_then(|l| github::latest(l))
        .and_then(|r| r.asset(kind))
        .cloned()
    else {
        return redirection(&page_github);
    };
    let disposition = format!("attachment; filename=\"{}\"", asset.name.replace('"', ""));
    let content_type = asset
        .content_type
        .clone()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "application/octet-stream".into());

    // 1. copie locale (reprise des téléchargements interrompus gérée par ServeFile)
    if let Some(chemin) = state.github.miroir.fichier(&asset.name, asset.size) {
        use tower::ServiceExt;
        let Ok(res) = tower_http::services::ServeFile::new(chemin).oneshot(req).await;
        let mut res = res.map(Body::new);
        for (name, value) in [(header::CONTENT_DISPOSITION, &disposition), (header::CONTENT_TYPE, &content_type)] {
            if let Ok(v) = HeaderValue::from_str(value) {
                res.headers_mut().insert(name, v);
            }
        }
        res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=300"));
        return res;
    }

    // 2. relais depuis GitHub
    let upstream = match state.github.miroir.http().get(&asset.browser_download_url).send().await {
        Ok(r) if r.status().is_success() => r,
        // 3. dernier recours : GitHub directement
        _ => return redirection(&asset.browser_download_url),
    };
    let longueur = upstream.content_length().unwrap_or(asset.size);
    let mut res = Response::new(Body::from_stream(upstream.bytes_stream()));
    let h = res.headers_mut();
    for (name, value) in [
        (header::CONTENT_TYPE, content_type),
        (header::CONTENT_LENGTH, longueur.to_string()),
        (header::CONTENT_DISPOSITION, disposition),
        (header::CACHE_CONTROL, "public, max-age=300".to_string()),
    ] {
        if let Ok(v) = HeaderValue::from_str(&value) {
            h.insert(name, v);
        }
    }
    res
}

fn redirection(url: &str) -> Response {
    let mut res = StatusCode::FOUND.into_response();
    if let Ok(v) = HeaderValue::from_str(url) {
        res.headers_mut().insert(header::LOCATION, v);
    }
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// `/status` : état des services dont Misty a besoin (vérifiés par le site toutes les minutes).
pub async fn status_page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<PageQuery>,
    headers: HeaderMap,
) -> Response {
    let (lang, remember) = i18n::negotiate(q.lang.as_deref(), &headers);
    let count = state.github.releases().await.map(|l| l.len());
    let p = page(&state, lang, "/status", count);
    let markup = status_view::render(&p, &state.status.lignes(), &state.status.incidents(), state.status.dispo_globale(), state.status.age());
    let mut res = html_response(StatusCode::OK, markup, lang, remember);
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// `/status.json` : la même chose pour les robots (et les curieux).
pub async fn status_json(State(state): State<Arc<AppState>>) -> Response {
    let services: Vec<_> = state
        .status
        .lignes()
        .iter()
        .map(|l| {
            let s = &crate::status::SERVICES[l.index];
            serde_json::json!({
                "name": s.nom,
                "url": s.url,
                "state": l.actuel.map(|m| m.etat.slug()),
                "latency_ms": l.actuel.and_then(|m| m.latence).map(|d| d.as_millis() as u64),
                "uptime_30d_percent": l.dispo,
                "days": l.jours.iter().map(|(d, j)| serde_json::json!({
                    "date": d.to_string(),
                    "state": j.map(|j| j.etat().slug()),
                    "uptime_percent": j.map(|j| j.dispo()),
                    "checks": j.map(|j| j.total()).unwrap_or(0),
                })).collect::<Vec<_>>(),
                "minutes": l.minutes.iter().map(|(t, m)| serde_json::json!({
                    "start": t,
                    "state": m.map(|m| m.etat.slug()),
                    "latency_ms": m.and_then(|m| m.ms),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let mut res = Json(serde_json::json!({
        "checked_seconds_ago": state.status.age(),
        "uptime_30d_percent": state.status.dispo_globale(),
        "incidents": state.status.incidents().iter().map(|i| serde_json::json!({
            "service": i.service,
            "kind": if i.panne { "outage" } else { "slowdown" },
            "start": i.debut,
            "end": i.fin,
        })).collect::<Vec<_>>(),
        "services": services,
    }))
    .into_response();
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

// ---------- Vérification Discord (captcha de Misty Bot) ----------

#[derive(Deserialize, Default)]
pub struct VerifyQuery {
    t: Option<String>,
    lang: Option<String>,
}

#[derive(Deserialize)]
pub struct VerifyForm {
    t: String,
    id: String,
    code: String,
    /// Champ piège : invisible, donc vide pour un humain
    #[serde(default)]
    website: String,
}

/// Langue : celle choisie sur le site (`?lang=`), sinon celle du serveur Discord (dans le lien).
fn verify_lang(q: &VerifyQuery, claims: Option<&crate::verify::Claims>, headers: &HeaderMap) -> (Lang, bool) {
    if q.lang.is_some() {
        return i18n::negotiate(q.lang.as_deref(), headers);
    }
    match claims.and_then(|c| Lang::from_code(&c.lang)) {
        Some(l) => (l, false),
        None => i18n::negotiate(None, headers),
    }
}

fn verify_response(state: &AppState, lang: Lang, remember: bool, etat: verify_view::Etat) -> Response {
    let p = page(state, lang, "/verify", None);
    let mut res = html_response(StatusCode::OK, verify_view::render(&p, etat), lang, remember);
    let h = res.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert("x-robots-tag", HeaderValue::from_static("noindex"));
    h.insert(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    res
}

/// Nouvelle image pour ce lien, ou la page « trop d'essais ».
fn captcha_or_limit(state: &AppState, lang: Lang, remember: bool, token: &str, erreur: bool) -> Response {
    match state.verification.nouveau_defi(token) {
        Some(defi) => verify_response(state, lang, remember, verify_view::Etat::Captcha { token, defi: &defi, erreur }),
        None => verify_response(state, lang, remember, verify_view::Etat::TropDEssais),
    }
}

/// `/verify?t=…` : le captcha.
pub async fn verify_page(State(state): State<Arc<AppState>>, Query(q): Query<VerifyQuery>, headers: HeaderMap) -> Response {
    let claims = q.t.as_deref().and_then(|t| state.verification.lien(t));
    let (lang, remember) = verify_lang(&q, claims.as_ref(), &headers);
    if !state.verification.active() {
        return verify_response(&state, lang, remember, verify_view::Etat::Indisponible);
    }
    match (claims, q.t.as_deref()) {
        (Some(_), Some(token)) => captcha_or_limit(&state, lang, remember, token, false),
        _ => verify_response(&state, lang, remember, verify_view::Etat::LienInvalide),
    }
}

/// Réponse au captcha : rôle donné si elle est juste.
pub async fn verify_submit(
    State(state): State<Arc<AppState>>,
    Query(q): Query<VerifyQuery>,
    headers: HeaderMap,
    axum::extract::Form(f): axum::extract::Form<VerifyForm>,
) -> Response {
    let claims = state.verification.lien(&f.t);
    let (lang, remember) = verify_lang(&q, claims.as_ref(), &headers);
    if !state.verification.active() {
        return verify_response(&state, lang, remember, verify_view::Etat::Indisponible);
    }
    let Some(claims) = claims else {
        return verify_response(&state, lang, remember, verify_view::Etat::LienInvalide);
    };
    if !f.website.is_empty() {
        return captcha_or_limit(&state, lang, remember, &f.t, true);
    }
    use crate::verify::Reponse;
    match state.verification.repondre(&f.id, &f.t, &f.code) {
        Reponse::Juste => match state.verification.donner_role(&claims).await {
            Ok(()) => verify_response(&state, lang, remember, verify_view::Etat::Reussi),
            Err(e) => {
                eprintln!("Vérification : rôle non donné (serveur {}) : {e}", claims.guild);
                verify_response(&state, lang, remember, verify_view::Etat::EchecRole)
            }
        },
        Reponse::Faux | Reponse::Perime => captcha_or_limit(&state, lang, remember, &f.t, true),
    }
}

/// `/verify/captcha/<id>.svg` : l'image d'un captcha.
pub async fn verify_captcha(State(state): State<Arc<AppState>>, Path(file): Path<String>) -> Response {
    let id = file.trim_end_matches(".svg");
    match state.verification.image(id) {
        Some(svg) => {
            let mut res = svg.into_response();
            let h = res.headers_mut();
            h.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"));
            h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            h.insert("x-robots-tag", HeaderValue::from_static("noindex"));
            res
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
