//! Routes HTTP : pages, API JSON et téléchargements.

use crate::AppState;
use crate::docs;
use crate::github::{self, Kind};
use crate::i18n::{self, Lang};
use crate::views::docs as docs_view;
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
    html_response(
        StatusCode::OK,
        home::render(&p, latest, &state.scene_home),
        lang,
        remember,
    )
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
    let markup = patch_notes::render(
        &p,
        releases.as_deref().map(|v| v.as_slice()),
        filter,
        &state.scene_notes,
    );
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

pub async fn not_found(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let (lang, _) = i18n::negotiate(None, &headers);
    let path = uri.path().to_string();
    let p = page(&state, lang, &path, None);
    html_response(
        StatusCode::NOT_FOUND,
        not_found::render(&p, &state.scene_notes),
        lang,
        false,
    )
}

/// Infos de la dernière version (même format que l'ancienne fonction Cloudflare).
pub async fn api_latest(State(state): State<Arc<AppState>>) -> Response {
    let releases = state.github.releases().await;
    let Some(r) = releases.as_deref().and_then(|l| github::latest(l)) else {
        return (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": "unavailable" })),
        )
            .into_response();
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

/// `/download/setup` et `/download/portable` : le fichier de la dernière version est relayé
/// depuis GitHub, l'utilisateur reste sur le domaine du site.
pub async fn download(State(state): State<Arc<AppState>>, Path(kind): Path<String>) -> Response {
    let Some(kind) = Kind::from_slug(&kind) else {
        return (StatusCode::NOT_FOUND, "Type inconnu").into_response();
    };
    let releases = state.github.releases().await;
    let Some(release) = releases.as_deref().and_then(|l| github::latest(l)) else {
        return (StatusCode::BAD_GATEWAY, "Release introuvable").into_response();
    };
    let Some(asset) = release.asset(kind) else {
        return (
            StatusCode::NOT_FOUND,
            "Fichier introuvable dans la dernière release",
        )
            .into_response();
    };
    let upstream = match state
        .github
        .http()
        .get(&asset.browser_download_url)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return (StatusCode::BAD_GATEWAY, "Téléchargement impossible").into_response(),
    };
    let content_type = asset
        .content_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".into());
    let disposition = format!("attachment; filename=\"{}\"", asset.name.replace('"', ""));
    let mut res = Response::new(Body::from_stream(upstream.bytes_stream()));
    let h = res.headers_mut();
    for (name, value) in [
        (header::CONTENT_TYPE, content_type),
        (header::CONTENT_LENGTH, asset.size.to_string()),
        (header::CONTENT_DISPOSITION, disposition),
        (header::CACHE_CONTROL, "public, max-age=300".to_string()),
    ] {
        if let Ok(v) = HeaderValue::from_str(&value) {
            h.insert(name, v);
        }
    }
    res
}
