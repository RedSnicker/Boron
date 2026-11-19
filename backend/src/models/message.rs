use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub message_type: MessageType,
    pub github_commit_sha: Option<String>,
    pub github_pr_number: Option<i32>,
    pub parent_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Default)]
#[sqlx(type_name = "VARCHAR(20)", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    #[default]
    Text,
    System,
    GithubCommit,
    GithubPr,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub channel_id: Uuid,
    pub content: String,
    pub message_type: Option<MessageType>,
    pub github_commit_sha: Option<String>,
    pub github_pr_number: Option<i32>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub display_name: String,
    pub content: String,
    pub message_type: MessageType,
    pub github_commit_sha: Option<String>,
    pub github_pr_number: Option<i32>,
    pub parent_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}