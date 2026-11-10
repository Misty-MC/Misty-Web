//! Site de Misty : serveur web en Rust (axum).
//!
//! Variables d'environnement :
//! - `PORT` (3000 par défaut)
//! - `GITHUB_TOKEN` (facultatif) : évite la limite de l'API GitHub
//! - `MISTY_REPO` (`Misty-MC/Misty` par défaut) : dépôt dont on affiche les releases
//! - `MISTY_PUBLIC_DIR` (`public` par défaut) : fichiers statiques (CSS, JS, images, polices)
//! - `SITE_URL` (`https://misty-launcher.online` par défaut) : adresse publique, pour le référencement
//! - `SKIN_API` (`https://mc-heads.net` par défaut) : service qui fournit les skins (boutique)
//! - `GITHUB_API` (`https://api.github.com` par défaut) : utile pour tester avec une fausse API
//! - `GITHUB_WEB` (`https://github.com` par défaut) : flux Atom et fichiers des releases
//! - `MISTY_CACHE_DIR` (dossier temporaire par défaut) : caches, statut, copie des versions, télémétrie
//! - `EARLY_BEFORE` (AAAA-MM-JJ, facultatif) : la cape early player de Misty Client est réservée aux
//!   comptes connectés avant cette date ; sans elle, tous les comptes y ont droit
//! - `VERIFY_SECRET` et `DISCORD_TOKEN` (facultatifs) : captcha `/verify` de Misty Bot (même secret
//!   que le bot, et le token du bot pour donner le rôle) ; `DISCORD_CLIENT_SECRET` en plus pour le
//!   captcha dans Discord (Activité, page `/activity`)
//! - `DISCORD_WEBHOOK` (facultatif) : webhook d'un salon privé de l'équipe, qui reçoit les rapports
//!   de logs et le résumé quotidien de la télémétrie
//! - `TELEMETRY_TOKEN` (facultatif, 16 caractères ou plus) : ouvre la lecture de la télémétrie
//!   (`/api/telemetry/summary`, `/api/telemetry/report/{code}`) avec `Authorization: Bearer …`

mod client;
mod discord;
mod puzzle;
mod verify;
mod docs;
mod format;
mod github;
mod miroir;
mod presence;
mod i18n;
mod legal;
mod markdown;
mod routes;
mod scene;
mod seedmap;
mod seo;
mod skins;
mod status;
mod telemetrie;
mod views;

use axum::Router;
use axum::http::{HeaderValue, header};
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::compression::predicate::{NotForContentType, Predicate};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

pub struct AppState {
    pub github: Arc<github::GitHub>,
    /// Page de statut : vérifications des services, faites en arrière-plan.
    pub status: Arc<status::Status>,
    /// Paysage de fond, le même sur toutes les pages.
    pub scene: String,
    /// Adresse publique du site, sans `/` final.
    pub site_url: String,
    /// Relais des skins pour l'aperçu 3D de la boutique.
    pub skins: skins::Skins,
    /// Bande de brume pixel, insérée entre les sections.
    pub fog_strip: String,
    /// Télémétrie du launcher et rapports de logs.
    pub telemetrie: Arc<telemetrie::Telemetrie>,
    /// Launchers ouverts en ce moment (/api/online).
    pub presence: presence::Presence,
    /// Joueurs en jeu avec Misty Client, et leurs cosmétiques Misty (/api/client/…).
    pub client: client::Client,
    /// Salon Discord de l'équipe (rapports, résumé du jour).
    pub discord: Arc<discord::Discord>,
    /// Captcha de vérification des serveurs Discord qui utilisent Misty Bot.
    pub verification: Arc<verify::Verification>,
}

/// Durée de cache des fichiers statiques. Une adresse avec `?v=` (CSS, JS, WebAssembly : voir
/// `views::layout::ASSET_VERSION`) change à chaque déploiement, son contenu ne change donc jamais :
/// le navigateur peut la garder un an sans redemander. Les autres gardent une heure.
async fn cache_statique(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
    let versionnee = req.uri().query().is_some_and(|q| q.starts_with("v=") || q.contains("&v="));
    let mut res = next.run(req).await;
    if versionnee && res.status().is_success() {
        res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=31536000, immutable"));
    }
    res
}

/// Client HTTP (GitHub, Mojang, Discord…), avec rustls et ring comme fournisseur de chiffrement
/// (installé une fois ; aws-lc, le fournisseur par défaut, est long à compiler).
pub fn http() -> reqwest::ClientBuilder {
    static TLS: std::sync::Once = std::sync::Once::new();
    TLS.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
    reqwest::Client::builder()
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[tokio::main]
async fn main() {
    i18n::init();
    docs::init();
    legal::init();

    let state = Arc::new(AppState {
        github: Arc::new(github::GitHub::new(
            env_or("GITHUB_API", "https://api.github.com"),
            env_or("GITHUB_WEB", "https://github.com"),
            env_or("MISTY_REPO", "Misty-MC/Misty"),
            std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty()),
        )),
        status: Arc::new(status::Status::new()),
        telemetrie: Arc::new(telemetrie::Telemetrie::new()),
        presence: presence::Presence::new(),
        client: client::Client::new(),
        discord: Arc::new(discord::Discord::new()),
        verification: Arc::new(verify::Verification::new()),
        scene: scene::render(&scene::SceneOptions {
            seed: 7,
            moon_x: 0.8,
            moon_r: 44,
        }),
        fog_strip: scene::fog_strip(11),
        skins: skins::Skins::new(env_or("SKIN_API", "https://mc-heads.net")),
        site_url: env_or("SITE_URL", "https://misty-launcher.online")
            .trim_end_matches('/')
            .to_string(),
    });

    // Tâches de fond : releases GitHub (les pages ne lisent que le cache) et page de statut
    tokio::spawn(state.github.clone().rafraichir_en_boucle());
    tokio::spawn(state.status.clone().verifier_en_boucle());
    tokio::spawn(state.telemetrie.clone().menage_en_boucle());
    tokio::spawn(state.discord.clone().resume_en_boucle(state.telemetrie.dossier()));

    // Fichiers statiques. Les versions .br et .gz préparées par le Dockerfile sont servies telles
    // quelles quand le navigateur les accepte : rien n'est compressé à chaque requête.
    let static_files = ServeDir::new(env_or("MISTY_PUBLIC_DIR", "public"))
        .precompressed_br()
        .precompressed_gzip()
        .not_found_service(get(routes::not_found).with_state(state.clone()));
    let static_files = tower::ServiceBuilder::new()
        .layer(axum::middleware::from_fn(cache_statique))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        ))
        .service(static_files);

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/patch-notes", get(routes::patch_notes))
        .route("/shop", get(routes::shop))
        .route("/skin/{name}", get(routes::skin))
        .route("/cape/{name}", get(routes::cape))
        .route("/player/{name}", get(routes::player))
        .route("/robots.txt", get(routes::robots))
        .route("/sitemap.xml", get(routes::sitemap))
        .route("/docs", get(routes::docs_index))
        .route("/docs/search.json", get(routes::docs_search_index))
        .route("/docs/{slug}", get(routes::docs_page))
        .route("/status", get(routes::status_page))
        .route("/tools", get(routes::tools_index))
        .route("/tools/seed", get(routes::tools_seed))
        .route("/legal", get(routes::legal_notice))
        .route("/privacy", get(routes::privacy))
        .route("/terms", get(routes::terms))
        .route("/status.json", get(routes::status_json))
        .route("/api/latest", get(routes::api_latest))
        .route("/download/{kind}", get(routes::download))
        .route("/verify", get(routes::verify_page))
        .route("/verify/answer", post(routes::verify_answer).layer(DefaultBodyLimit::max(32 * 1024)))
        .route("/verify/new", post(routes::verify_new))
        .route("/verify/captcha/{id}/{file}", get(routes::verify_image))
        .route("/activity", get(routes::activity_page))
        .route("/activity/start", post(routes::activity_start))
        .route("/activity/log", post(routes::activity_log).layer(DefaultBodyLimit::max(2 * 1024)))
        .route("/api/verify/config", post(routes::verify_config).layer(DefaultBodyLimit::max(4 * 1024)))
        .route(
            "/api/presence",
            post(presence::signe).layer(DefaultBodyLimit::max(2 * 1024)),
        )
        .route("/api/online", get(presence::en_ligne))
        .route("/api/client/login", post(client::login).layer(DefaultBodyLimit::max(1024)))
        .route("/api/client/presence", post(client::presence).layer(DefaultBodyLimit::max(1024)))
        .route("/api/client/players", post(client::joueurs).layer(DefaultBodyLimit::max(8 * 1024)))
        .route("/api/client/early/{uuid}", get(client::early))
        .route(
            "/api/telemetry",
            post(telemetrie::evenement).layer(DefaultBodyLimit::max(32 * 1024)),
        )
        .route(
            "/api/telemetry/report",
            post(telemetrie::rapport).layer(DefaultBodyLimit::max(1024 * 1024)),
        )
        .route("/api/telemetry/summary", get(telemetrie::resume))
        .route("/api/telemetry/report/{code}", get(telemetrie::lire_rapport))
        .fallback_service(static_files)
        // les téléchargements (zip, exe) sont déjà compressés : sans recompression, leur taille
        // reste annoncée (barre de progression du navigateur et de la mise à jour du launcher)
        .layer(CompressionLayer::new().compress_when(
            tower_http::compression::DefaultPredicate::new()
                .and(NotForContentType::const_new("application/zip"))
                .and(NotForContentType::const_new("application/octet-stream"))
                .and(NotForContentType::const_new("application/vnd.microsoft.portable-executable"))
                .and(NotForContentType::const_new("application/x-msdownload")),
        ))
        .with_state(state);

    let port = env_or("PORT", "3000");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("port déjà utilisé ?");
    println!("Misty-site sur http://localhost:{port}");
    axum::serve(listener, app).await.expect("serveur");
}
