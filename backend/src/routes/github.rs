use axum::{
    routing::{get, post},
    Json, Router,
};
use crate::models::github_integration::{GithubRepoConfig, GithubCommit, GithubPullRequest};
use std::sync::Arc;

pub fn routes(pool: sqlx::SqlitePool) -> Router {
    Router::new()
        .route("/connect", post(connect_github))
        .route("/repos/:team_id", get(list_repos))
        .route("/commits/:team_id", get(get_commits))
        .route("/pulls/:team_id", get(get_pull_requests))
        .route("/webhook", post(webhook))
        .with_state(pool)
}

async fn connect_github(
    Json(payload): Json<GithubRepoConfig>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    Ok(Json(serde_json::json!({
        "status": "connected",
        "repo": format!("{}/{}", payload.repo_owner, payload.repo_name),
    })))
}

async fn list_repos(
    axum::extract::Path(team_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<Vec<serde_json::Value>>, axum::response::Response> {
    Ok(Json(vec![
        serde_json::json!({
            "id": 1,
            "name": "boron-web",
            "full_name": "team/boron-web",
            "description": "Frontend for Boron team app"
        }),
    ]))
}

async fn get_commits(
    axum::extract::Path(team_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<Vec<GithubCommit>>, axum::response::Response> {
    Ok(Json(vec![
        GithubCommit {
            sha: "a1b2c3d4".to_string(),
            message: "Fix login bug".to_string(),
            author: "developer".to_string(),
            url: "https://github.com/team/boron-web/commit/a1b2c3d4".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ]))
}

async fn get_pull_requests(
    axum::extract::Path(team_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<Vec<GithubPullRequest>>, axum::response::Response> {
    Ok(Json(vec![
        GithubPullRequest {
            number: 42,
            title: "Add issues tab".to_string(),
            state: "open".to_string(),
            url: "https://github.com/team/boron-web/pull/42".to_string(),
            author: "developer".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    ]))
}

async fn webhook(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, axum::response::Response> {
    Ok(Json(serde_json::json!({"status": "received"})))
}