//! Site de Misty : serveur web en Rust (axum).
//!
//! Variables d'environnement :
//! - `PORT` (3000 par défaut)
//! - `GITHUB_TOKEN` (facultatif) : évite la limite de l'API GitHub
//! - `MISTY_REPO` (`Misty-MC/Misty` par défaut) : dépôt dont on affiche les releases
//! - `MISTY_PUBLIC_DIR` (`public` par défaut) : fichiers statiques (CSS, JS, images, polices)
//! - `GITHUB_API` (`https://api.github.com` par défaut) : utile pour tester avec une fausse API

mod docs;
mod format;
mod github;
mod i18n;
mod markdown;
mod routes;
mod scene;
mod views;

use axum::Router;
use axum::http::{HeaderValue, header};
use axum::routing::get;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

pub struct AppState {
    pub github: github::GitHub,
    /// Paysage de fond, le même sur toutes les pages.
    pub scene: String,
    /// Bande de brume pixel, insérée entre les sections.
    pub fog_strip: String,
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

    let state = Arc::new(AppState {
        github: github::GitHub::new(
            env_or("GITHUB_API", "https://api.github.com"),
            env_or("MISTY_REPO", "Misty-MC/Misty"),
            std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty()),
        ),
        scene: scene::render(&scene::SceneOptions {
            seed: 7,
            moon_x: 0.8,
            moon_r: 44,
        }),
        fog_strip: scene::fog_strip(11),
    });

    let static_files = ServeDir::new(env_or("MISTY_PUBLIC_DIR", "public"))
        .not_found_service(get(routes::not_found).with_state(state.clone()));
    let static_files = tower::ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        ))
        .service(static_files);

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/patch-notes", get(routes::patch_notes))
        .route("/shop", get(routes::shop))
        .route("/docs", get(routes::docs_index))
        .route("/docs/search.json", get(routes::docs_search_index))
        .route("/docs/{slug}", get(routes::docs_page))
        .route("/api/latest", get(routes::api_latest))
        .route("/download/{kind}", get(routes::download))
        .fallback_service(static_files)
        .layer(CompressionLayer::new())
        .with_state(state);

    let port = env_or("PORT", "3000");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("port déjà utilisé ?");
    println!("Misty-site sur http://localhost:{port}");
    axum::serve(listener, app).await.expect("serveur");
}
