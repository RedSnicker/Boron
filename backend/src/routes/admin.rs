use axum::{routing::post, Json, Router, extract::{State, Extension}};
use crate::models::error::AppError;
use axum::http::StatusCode;
use uuid::Uuid;
use serde::Deserialize;

#[derive(Deserialize)]
struct AdminCommand { command: String, args: Option<Vec<String>> }

pub fn routes(pool: sqlx::SqlitePool) -> Router {
    Router::new()
        .route("/command", post(execute_command))
        .with_state(pool)
}

async fn execute_command(
    State(pool): State<sqlx::SqlitePool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<AdminCommand>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify admin
    let role: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE id=?")
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: "Database error".into() })?;
    
    let role = role.ok_or_else(|| AppError { status: StatusCode::NOT_FOUND, message: "User not found".into() })?;
    if role != "admin" && role != "owner" {
        return Err(AppError { status: StatusCode::FORBIDDEN, message: "Admin access required".into() });
    }

    let cmd = payload.command.trim();
    let output = match cmd {
        "status" => {
            let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
                .fetch_one(&pool).await.unwrap_or(0);
            let team_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM teams")
                .fetch_one(&pool).await.unwrap_or(0);
            let channel_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM channels")
                .fetch_one(&pool).await.unwrap_or(0);
            format!("Users: {}\nTeams: {}\nChannels: {}", user_count, team_count, channel_count)
        }
        "users" => {
            let rows = sqlx::query_as::<_, (Uuid, String, String, String, bool)>(
                "SELECT id, email, display_name, role, is_active FROM users ORDER BY display_name"
            ).fetch_all(&pool).await.map_err(|_| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: "Query failed".into() })?;
            let output: String = rows.iter().map(|(id, email, name, role, active)| {
                format!("{} | {} | {} | {} | {}", id, email, name, role, if *active { "active" } else { "inactive" })
            }).collect::<Vec<_>>().join("\n");
            output
        }
        "teams" => {
            let rows = sqlx::query_as::<_, (Uuid, String, String)>(
                "SELECT id, name, slug FROM teams ORDER BY name"
            ).fetch_all(&pool).await.map_err(|_| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: "Query failed".into() })?;
            let output: String = rows.iter().map(|(id, name, slug)| {
                format!("{} | {} | {}", id, name, slug)
            }).collect::<Vec<_>>().join("\n");
            output
        }
        "help" => {
            "Available commands:\n  status  - Show server statistics\n  users   - List all users\n  teams   - List all teams\n  help    - Show this help\n".to_string()
        }
        _ => return Err(AppError { status: StatusCode::BAD_REQUEST, message: format!("Unknown command: {}", cmd) }),
    };

    Ok(Json(serde_json::json!({"output": output})))
}