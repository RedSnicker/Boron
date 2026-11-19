use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GithubIntegration {
    pub id: Uuid,
    pub team_id: Uuid,
    pub installation_id: i64,
    pub repo_owner: String,
    pub repo_name: String,
    pub access_token_encrypted: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct GithubRepoConfig {
    pub team_id: Uuid,
    pub repo_owner: String,
    pub repo_name: String,
}

#[derive(Debug, Serialize)]
pub struct GithubCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub url: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct GithubPullRequest {
    pub number: i32,
    pub title: String,
    pub state: String,
    pub url: String,
    pub author: String,
    pub created_at: String,
}