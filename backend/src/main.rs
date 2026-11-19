mod access;
mod storage;
mod middleware;
mod models;
mod routes;
mod services;

use axum::{
    routing::get,
    Router,
    http::{HeaderValue, Uri, header, StatusCode},
    response::Response,
};
use crate::models::error::AppError;
use std::{net::SocketAddr, str::FromStr};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct FrontendAssets;

async fn serve_frontend(uri: Uri) -> Result<Response, AppError> {
    let path = uri.path().trim_start_matches('/');

    if !path.is_empty() {
        if let Some(content) = FrontendAssets::get(path) {
            let mime = mime_for_path(path);
            return Ok(Response::builder()
                .header(header::CONTENT_TYPE, mime)
                .body(content.data.into())
                .unwrap());
        }
    }

    let index = FrontendAssets::get("index.html")
        .ok_or_else(|| AppError { status: StatusCode::NOT_FOUND, message: "index.html not found".into() })?;
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(index.data.into())
        .unwrap())
}

fn mime_for_path(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::dotenv().ok();

    // Security: reject default/fallback JWT keys at startup
    let jwt_secret = std::env::var("JWT_SECRET");
    if let Ok(ref secret) = jwt_secret {
        if secret == "default_secret_change_me" || secret == "change_this_to_a_secure_random_string_in_production" || secret.is_empty() {
            eprintln!("CRITICAL SECURITY ERROR: JWT_SECRET is set to a default/fallback value. Generate a strong random secret with: openssl rand -hex 32");
            std::process::exit(1);
        }
    } else {
        eprintln!("CRITICAL SECURITY ERROR: JWT_SECRET environment variable is not set. Generate one with: openssl rand -hex 32");
        std::process::exit(1);
    }
    let jwt_secret = jwt_secret.unwrap();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let options = SqliteConnectOptions::from_str(&database_url)
        .expect("DATABASE_URL must be a valid SQLite URL")
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = match SqlitePoolOptions::new().connect_with(options).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            tracing::error!("Please ensure DATABASE_URL is correct and its parent directory is writable");
            std::process::exit(1);
        }
    };

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let cors = CorsLayer::new()
        .allow_origin(
            std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:57575".into())
                .parse::<HeaderValue>()
                .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:57575")),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    async fn api_version() -> axum::Json<serde_json::Value> { axum::Json(serde_json::json!({"api_revision": 7, "service":"boron-backend"})) }
    async fn health_check() -> Result<&'static str, AppError> {
        Ok("OK")
    }

    let app = Router::new()
        .with_state(pool.clone())
        .route("/health", get(health_check))
        .route("/api/version", get(api_version))
        .merge(Router::new().route("/api/avatars/:id", get(storage::avatar)).route("/api/team-avatars/:id", get(storage::team_avatar)).route("/api/files/:id/ticket", get(storage::ticket)).route("/api/file-download/:token", get(storage::ticket_download)).with_state(pool.clone()))
        .nest("/api/auth", routes::auth::routes(pool.clone()))
        .nest("/api/teams", routes::teams::routes(pool.clone()))
        .nest("/api/channels", routes::channels::routes(pool.clone()))
        .merge(Router::new().route("/api/reminders", get(routes::chat::list_reminders)).route("/api/reminders/:id/delivered", axum::routing::post(routes::chat::deliver_reminder)).with_state(pool.clone()))
        .nest("/api/stickers", routes::stickers::routes(pool.clone()))
        .nest("/api/issues", routes::issues::routes(pool.clone()))
        .nest("/api/notes", routes::notes::routes(pool.clone()))
        .nest("/api/github", routes::github::routes(pool.clone()))
        .nest("/api/admin", routes::admin::routes(pool.clone()))
        .route("/ws", get(routes::websocket::handler))
        .fallback(serve_frontend)
        .layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer(cors);

    // Port resolution: check WEB_PORT, then PORT, then SERVER_PORT (Pterodactyl/Altare), default 3001
    let web_port = std::env::var("WEB_PORT")
        .or_else(|_| std::env::var("PORT"))
        .or_else(|_| std::env::var("SERVER_PORT"))
        .unwrap_or_else(|_| {
            tracing::warn!("No WEB_PORT/PORT/SERVER_PORT env var found, defaulting to 3001");
            "3001".into()
        });
    let port_source = if std::env::var("WEB_PORT").is_ok() { "WEB_PORT" }
        else if std::env::var("PORT").is_ok() { "PORT" }
        else if std::env::var("SERVER_PORT").is_ok() { "SERVER_PORT" }
        else { "default" };
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let addr: SocketAddr = format!("{}:{}", host, web_port)
        .parse()
        .expect("Invalid socket address");

    tracing::info!("Server listening on {} (port from {}={})", addr, port_source, web_port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}