use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Issue {
    pub id: Uuid,
    pub team_id: Uuid,
    pub channel_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Uuid,
    pub github_issue_number: Option<i32>,
    pub github_pr_number: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR(20)", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Open,
    InProgress,
    Review,
    Done,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR(20)", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IssuePriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Deserialize)]
pub struct CreateIssueRequest {
    pub team_id: Uuid,
    pub channel_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<IssuePriority>,
    pub assignee_id: Option<Uuid>,
    pub assignee_ids: Option<Vec<Uuid>>,
}