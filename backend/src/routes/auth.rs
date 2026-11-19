use axum::{
    routing::{get, post},
    extract::{Multipart, DefaultBodyLimit},
    Json, Router, extract::{State, Extension},
};
use crate::models::error::AppError;
use crate::models::user::{
    CreateUserRequest, LoginRequest, UserResponse, AuthResponse,
};
use crate::services::auth_service::AuthService;
use axum::http::StatusCode;
use uuid::Uuid;

pub fn routes(pool: sqlx::SqlitePool) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(get_me).put(update_me))
        .route("/avatar", post(upload_avatar).layer(DefaultBodyLimit::max(crate::storage::MAX_REQUEST)))
        .with_state(pool)
}

async fn register(
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let service = AuthService::new(pool);
    service.register(payload).await
        .map(Json)
        .map_err(|e| AppError { status: e.status_code, message: e.message })
}

async fn login(
    State(pool): State<sqlx::SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let service = AuthService::new(pool);
    service.login(payload).await
        .map(Json)
        .map_err(|e| AppError { status: e.status_code, message: e.message })
}

async fn get_me(
    State(pool): State<sqlx::SqlitePool>,
    axum::extract::Extension(user_id): Extension<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let service = AuthService::new(pool);
    service.get_user(user_id).await
        .map(|u| Json(UserResponse::from(u)))
        .map_err(|e| AppError { status: e.status_code, message: e.message })
}
async fn upload_avatar(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, multipart:Multipart) -> Result<Json<serde_json::Value>,AppError> {
 let upload=crate::storage::receive(multipart,true).await?;
 crate::storage::store(&pool,"avatar",user,user,&upload).await?;
 let url=format!("/api/avatars/{}?v={}",user,upload.id);
 sqlx::query("UPDATE users SET avatar_url=? WHERE id=?").bind(&url).bind(user).execute(&pool).await?;
 let old:Vec<Uuid>=sqlx::query_scalar("SELECT id FROM stored_files WHERE scope='avatar' AND parent_id=? AND id<>?").bind(user).bind(upload.id).fetch_all(&pool).await?;
 for id in old {sqlx::query("DELETE FROM stored_files WHERE id=?").bind(id).execute(&pool).await?;let _=tokio::fs::remove_file(crate::storage::path(id)).await;}
 Ok(Json(serde_json::json!({"avatar_url":url})))
}

#[derive(serde::Deserialize)] struct ProfileUpdate { display_name: String, bio: String }
async fn update_me(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Json(p):Json<ProfileUpdate>) -> Result<Json<UserResponse>,AppError> {
 let name=p.display_name.trim(); if name.is_empty()||name.chars().count()>100||p.bio.chars().count()>2000 {return Err(AppError{status:StatusCode::BAD_REQUEST,message:"Name is required (up to 100 characters); bio can be up to 2,000 characters.".into()});}
 sqlx::query("UPDATE users SET display_name=?, bio=?, updated_at=? WHERE id=?").bind(name).bind(&p.bio).bind(chrono::Utc::now().naive_utc()).bind(user).execute(&pool).await?;
 let service=AuthService::new(pool);service.get_user(user).await.map(|u|Json(UserResponse::from(u))).map_err(|e|AppError{status:e.status_code,message:e.message})
}
