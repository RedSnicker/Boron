use axum::{routing::{get, post, put, delete}, Json, Router, extract::{State, Extension, Path}};
use crate::models::error::AppError;
use axum::http::StatusCode;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Role {
    pub id: i32,
    pub team_id: Uuid,
    pub name: String,
    pub color: String,
    pub permissions: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub team_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub permissions: Option<Vec<String>>,
}

pub fn routes(pool: sqlx::SqlitePool) -> Router {
    Router::new()
        .route("/", get(list_roles).post(create_role))
        .route("/:id", put(update_role).delete(delete_role))
        .with_state(pool)
}

async fn list_roles(
    State(pool): State<sqlx::SqlitePool>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, Uuid>>,
) -> Result<Json<Vec<Role>>, AppError> {
    let team_id = params.get("team_id").copied().unwrap_or_else(|| Uuid::new_v4());
    let rows = sqlx::query_as::<_, (i32, Uuid, String, String, String)>(
        "SELECT id, team_id, name, color, permissions FROM roles WHERE team_id = ? ORDER BY id"
    ).bind(team_id).fetch_all(&pool).await.map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: e.to_string() })?;
    Ok(Json(rows.into_iter().map(|(id, team_id, name, color, permissions)| {
        let perms: Vec<String> = serde_json::from_str(&permissions).unwrap_or_default();
        Role { id, team_id, name, color, permissions: perms }
    }).collect()))
}

async fn create_role(
    State(pool): State<sqlx::SqlitePool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<Json<Role>, AppError> {
    let role: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
        .bind(user_id).fetch_optional(&pool).await.ok().flatten();
    if role.as_deref() != Some("admin") && role.as_deref() != Some("owner") {
        return Err(AppError { status: StatusCode::FORBIDDEN, message: "Admin required".into() });
    }
    let now = chrono::Utc::now().naive_utc().to_string();
    let perms = payload.permissions.unwrap_or_default();
    let perms_json = serde_json::to_string(&perms).unwrap_or_default();
    let color = payload.color.unwrap_or_else(|| "#7289da".to_string());
    sqlx::query("INSERT INTO roles (team_id, name, color, permissions, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(payload.team_id).bind(&payload.name).bind(&color).bind(&perms_json).bind(&now)
        .execute(&pool).await.map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: e.to_string() })?;
    let id: i32 = sqlx::query_scalar("SELECT last_insert_rowid()")
        .fetch_one(&pool).await.map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: e.to_string() })?;
    Ok(Json(Role { id, team_id: payload.team_id, name: payload.name, color, permissions: perms }))
}

async fn update_role(
    State(pool): State<sqlx::SqlitePool>,
    Extension(user_id): Extension<Uuid>,
    Path(role_id): Path<i32>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<Json<Role>, AppError> {
    let role: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
        .bind(user_id).fetch_optional(&pool).await.ok().flatten();
    if role.as_deref() != Some("admin") && role.as_deref() != Some("owner") {
        return Err(AppError { status: StatusCode::FORBIDDEN, message: "Admin required".into() });
    }
    let perms = payload.permissions.clone().unwrap_or_default();
    let perms_json = serde_json::to_string(&perms).unwrap_or_default();
    let color = payload.color.unwrap_or_else(|| "#7289da".to_string());
    sqlx::query("UPDATE roles SET name = ?, color = ?, permissions = ? WHERE id = ?")
        .bind(&payload.name).bind(&color).bind(&perms_json).bind(role_id)
        .execute(&pool).await.map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: e.to_string() })?;
    Ok(Json(Role { id: role_id, team_id: payload.team_id, name: payload.name, color, permissions: perms }))
}

async fn delete_role(
    State(pool): State<sqlx::SqlitePool>,
    Extension(user_id): Extension<Uuid>,
    Path(role_id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let role: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
        .bind(user_id).fetch_optional(&pool).await.ok().flatten();
    if role.as_deref() != Some("admin") && role.as_deref() != Some("owner") {
        return Err(AppError { status: StatusCode::FORBIDDEN, message: "Admin required".into() });
    }
    sqlx::query("DELETE FROM roles WHERE id = ?").bind(role_id).execute(&pool).await.map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: e.to_string() })?;
    Ok(Json(serde_json::json!({"status": "deleted"})))
}