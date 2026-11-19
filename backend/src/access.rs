use crate::models::error::AppError;
use axum::http::StatusCode;
use sqlx::SqlitePool;
use uuid::Uuid;

fn denied() -> AppError { AppError { status: StatusCode::FORBIDDEN, message: "You do not have access to this workspace.".into() } }

pub async fn team_member(pool: &SqlitePool, team: Uuid, user: Uuid) -> Result<(), AppError> {
    let found: i64 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = ? AND user_id = ?)")
        .bind(team).bind(user).fetch_one(pool).await?;
    if found == 1 { Ok(()) } else { Err(denied()) }
}
pub async fn team_admin(pool: &SqlitePool, team: Uuid, user: Uuid) -> Result<(), AppError> {
    let found: i64 = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = ? AND user_id = ? AND role IN ('owner', 'admin'))")
        .bind(team).bind(user).fetch_one(pool).await?;
    if found == 1 { Ok(()) } else { Err(denied()) }
}
pub async fn channel_access(pool: &SqlitePool, channel: Uuid, user: Uuid) -> Result<Uuid, AppError> {
    let team: Option<Uuid> = sqlx::query_scalar("SELECT c.team_id FROM channels c JOIN team_members tm ON tm.team_id = c.team_id AND tm.user_id = ? WHERE c.id = ? AND (c.is_private = 0 OR tm.role IN ('owner','admin') OR EXISTS (SELECT 1 FROM channel_members cm WHERE cm.channel_id = c.id AND cm.user_id = ?))")
        .bind(user).bind(channel).bind(user).fetch_optional(pool).await?;
    team.ok_or_else(denied)
}
