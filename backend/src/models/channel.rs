use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub slug: String,
    pub r#type: ChannelType,
    pub description: Option<String>,
    pub is_private: bool,
    pub required_role: Option<ChannelRole>,
    pub position: i32,
    pub icon: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR(20)", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Text,
    Issues,
    Notes,
    Page,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "VARCHAR(20)", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChannelRole {
    Admin,
    Member,
    Viewer,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub team_id: Uuid,
    pub name: String,
    pub slug: String,
    pub r#type: ChannelType,
    pub description: Option<String>,
    pub is_private: Option<bool>,
    pub required_role: Option<ChannelRole>,
    pub position: Option<i32>,
    pub icon: Option<String>,
}