use axum::{body::Body, extract::State, http::{Request, StatusCode, Method}, middleware::Next, response::Response};
use uuid::Uuid;

pub async fn auth_middleware(State(pool): State<sqlx::SqlitePool>, mut request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    // Frontend static files (served on /) and websocket do not need API auth.
    // Websocket handles its own auth via Bearer token in the upgrade request.
    if path == "/health" || path == "/ws" || (!path.starts_with("/api") && path != "/api") {
        return Ok(next.run(request).await);
    }
    if request.method() == Method::OPTIONS || path == "/api/version" || path == "/api/auth/login" || path == "/api/auth/register" || (request.method() == Method::GET && (path.starts_with("/api/avatars/") || path.starts_with("/api/team-avatars/") || path.starts_with("/api/file-download/"))) {
        return Ok(next.run(request).await);
    }
    let token = request.headers().get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")).ok_or(StatusCode::UNAUTHORIZED)?;
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_change_me".into());
    let claims: jsonwebtoken::TokenData<serde_json::Value> = jsonwebtoken::decode(token, &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()), &jsonwebtoken::Validation::default()).map_err(|_| StatusCode::UNAUTHORIZED)?;
    if claims.claims.get("kind").and_then(|v| v.as_str()) == Some("file_download") { return Err(StatusCode::UNAUTHORIZED); }
    let user_id = claims.claims.get("sub").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()).ok_or(StatusCode::UNAUTHORIZED)?;
    let active: Option<bool> = sqlx::query_scalar("SELECT is_active FROM users WHERE id = ?").bind(user_id).fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if active != Some(true) { return Err(StatusCode::UNAUTHORIZED); }
    request.extensions_mut().insert(user_id);
    Ok(next.run(request).await)
}
