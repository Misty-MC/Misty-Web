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
    uri: Uri,
) -> Response {
    // Ouvert comme Activité Discord (`<application>.discordsays.com/?instance_id=…&frame_id=…`) :
    // direction la vérification, en gardant les paramètres que lit le SDK Discord.
    if let Some(query) = uri.query()
        && query.contains("frame_id=")
        && query.contains("instance_id=")
    {
        return redirection(&format!("/activity?{query}"));
    }
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

// ---------- Vérification Discord (captcha puzzle de Misty Bot) ----------

#[derive(Deserialize, Default)]
pub struct VerifyQuery {
    t: Option<String>,
    lang: Option<String>,
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

fn no_store(mut res: Response) -> Response {
    let h = res.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert("x-robots-tag", HeaderValue::from_static("noindex"));
    h.insert(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    res
}

/// `/verify?t=…` : consentement, puis le puzzle (tout se joue ensuite en JSON, sans recharger).
pub async fn verify_page(State(state): State<Arc<AppState>>, Query(q): Query<VerifyQuery>, headers: HeaderMap) -> Response {
    let claims = q.t.as_deref().and_then(|t| state.verification.lien(t));
    let (lang, remember) = verify_lang(&q, claims.as_ref(), &headers);
    let p = page(&state, lang, "/verify", None);
    let v = &state.verification;
    let etat = if !v.active() {
        verify_view::Etat::Indisponible
    } else {
        match (&claims, q.t.as_deref()) {
            (Some(c), Some(_)) if v.a_le_role(c.guild, c.user, c.role).await == Some(true) => {
                verify_view::Etat::DejaVerifie
            }
            (Some(c), Some(token)) => match v.nouveau_defi(token) {
                Some((captcha, y)) => verify_view::Etat::Puzzle {
                    token: token.to_string(),
                    captcha,
                    y,
                    server: v.config(&c.guild.to_string()).and_then(|c| c.name),
                },
                None => verify_view::Etat::TropDEssais,
            },
            _ => verify_view::Etat::LienInvalide,
        }
    };
    no_store(html_response(StatusCode::OK, verify_view::render(&p, etat), lang, remember))
}

fn erreur(code: &str) -> Response {
    no_store(Json(serde_json::json!({ "error": code })).into_response())
}

#[derive(Deserialize)]
pub struct VerifyAnswer {
    t: String,
    id: String,
    /// Position finale de la pièce, en pixels de l'image
    x: f32,
    /// Geste : (temps en ms, x)
    #[serde(default)]
    trace: Vec<(f32, f32)>,
}

/// Réponse au puzzle : rôle donné si elle est juste, sinon un nouveau puzzle.
pub async fn verify_answer(State(state): State<Arc<AppState>>, Json(b): Json<VerifyAnswer>) -> Response {
    let v = &state.verification;
    let Some(claims) = v.lien(&b.t) else {
        return erreur("expired");
    };
    let trace: Vec<(f32, f32)> = b.trace.into_iter().take(400).collect();
    use crate::verify::Reponse;
    match v.repondre(&b.id, &b.t, b.x, &trace) {
        Reponse::Juste => match v.donner_role(&claims).await {
            Ok(()) => no_store(Json(serde_json::json!({ "ok": true })).into_response()),
            Err(e) => {
                eprintln!("Vérification : rôle non donné (serveur {}) : {e}", claims.guild);
                erreur("role")
            }
        },
        Reponse::Faux | Reponse::Perime => match v.nouveau_defi(&b.t) {
            Some((captcha, y)) => no_store(Json(serde_json::json!({ "ok": false, "captcha": captcha, "y": y })).into_response()),
            None => erreur("many"),
        },
    }
}

#[derive(Deserialize)]
pub struct VerifyNew {
    t: String,
}

/// « Autre défi » : un nouveau puzzle pour le même lien.
pub async fn verify_new(State(state): State<Arc<AppState>>, Json(b): Json<VerifyNew>) -> Response {
    if state.verification.lien(&b.t).is_none() {
        return erreur("expired");
    }
    match state.verification.nouveau_defi(&b.t) {
        Some((captcha, y)) => no_store(Json(serde_json::json!({ "captcha": captcha, "y": y })).into_response()),
        None => erreur("many"),
    }
}

/// `/verify/captcha/<id>/bg.png` (fond avec le trou) et `/verify/captcha/<id>/piece.png`.
pub async fn verify_image(State(state): State<Arc<AppState>>, Path((id, file)): Path<(String, String)>) -> Response {
    let piece = match file.as_str() {
        "bg.png" => false,
        "piece.png" => true,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let st = state.clone();
    let png = tokio::task::spawn_blocking(move || st.verification.image(&id, piece)).await.ok().flatten();
    match png {
        Some(png) => {
            let mut res = png.into_response();
            res.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
            no_store(res)
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// ---------- Vérification dans Discord (Activité) ----------

#[derive(Deserialize, Default)]
pub struct ActivityQuery {
    guild_id: Option<String>,
    lang: Option<String>,
}

/// `/activity` : la page ouverte dans Discord.
pub async fn activity_page(State(state): State<Arc<AppState>>, Query(q): Query<ActivityQuery>, headers: HeaderMap) -> Response {
    let config = q.guild_id.as_deref().and_then(|g| state.verification.config(g));
    // langue du serveur Discord (envoyée par le bot), sinon celle du navigateur
    let lang = q
        .lang
        .as_deref()
        .and_then(Lang::from_code)
        .or_else(|| config.as_ref().and_then(|c| Lang::from_code(&c.lang)))
        .unwrap_or_else(|| i18n::negotiate(None, &headers).0);
    let p = page(&state, lang, "/activity", None);
    let client = state.verification.activite_active().then(|| state.verification.client_id()).flatten();
    let server = config.and_then(|c| c.name);
    no_store(html_response(StatusCode::OK, verify_view::activite(&p, client, server.as_deref()), lang, false))
}

#[derive(Deserialize)]
pub struct ActivityStart {
    code: String,
    guild_id: Option<String>,
    /// Salon où l'Activité a été ouverte
    #[serde(default)]
    channel_id: Option<String>,
}

/// Début : qui est le membre (code OAuth2), lien signé pour son serveur, premier puzzle.
pub async fn activity_start(State(state): State<Arc<AppState>>, Json(b): Json<ActivityStart>) -> Response {
    let v = &state.verification;
    if !v.activite_active() {
        return erreur("off");
    }
    let Some(guild) = b.guild_id.filter(|g| !g.is_empty()) else {
        return erreur("guild");
    };
    // seulement dans le salon du règlement (pas depuis le lanceur d'Activités d'un autre salon)
    if let Some(rules) = v.config(&guild).and_then(|c| c.channel)
        && b.channel_id.as_deref() != Some(rules.as_str())
    {
        return erreur("channel");
    }
    let user = match v.membre(&b.code).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Activité : autorisation Discord refusée : {e}");
            return erreur("auth");
        }
    };
    let Some(token) = v.lien_pour(&guild, user) else {
        return erreur("guild");
    };
    // déjà vérifié : rien à refaire
    if let Some(c) = v.lien(&token)
        && v.a_le_role(c.guild, c.user, c.role).await == Some(true)
    {
        return erreur("already");
    }
    match v.nouveau_defi(&token) {
        Some((captcha, y)) => no_store(Json(serde_json::json!({ "token": token, "captcha": captcha, "y": y })).into_response()),
        None => erreur("many"),
    }
}

/// `/api/verify/config` : le bot envoie la config de vérification d'un serveur (signée).
pub async fn verify_config(State(state): State<Arc<AppState>>, headers: HeaderMap, body: axum::body::Bytes) -> Response {
    let sig = headers.get("x-misty-signature").and_then(|v| v.to_str().ok()).unwrap_or("");
    if state.verification.recevoir_config(&body, sig) {
        StatusCode::NO_CONTENT.into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[derive(Deserialize)]
pub struct ActivityLog {
    detail: String,
}

/// Erreur rencontrée dans Discord par la page `/activity` (notée dans les logs du site).
pub async fn activity_log(Json(b): Json<ActivityLog>) -> StatusCode {
    let detail: String = b.detail.chars().filter(|c| !c.is_control()).take(300).collect();
    eprintln!("Activité (navigateur) : {detail}");
    StatusCode::NO_CONTENT
}
