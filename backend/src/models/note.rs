use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Note {
    pub id: Uuid,
    pub team_id: Uuid,
    pub channel_id: Uuid,
    pub title: String,
    pub content: String,
    pub color: String,
    pub position_x: i32,
    pub position_y: i32,
    pub author_id: Uuid,
    pub is_pinned: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub team_id: Uuid,
    pub channel_id: Uuid,
    pub title: String,
    pub content: String,
    pub color: Option<String>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<String>,
    pub position_x: Option<i32>,
    pub position_y: Option<i32>,
    pub is_pinned: Option<bool>,
}