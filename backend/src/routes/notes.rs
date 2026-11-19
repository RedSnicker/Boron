use axum::{
    routing::{post, put, delete},
    Json, Router, extract::{State, Extension, Path, Query},
};
use crate::models::error::AppError;
use crate::access;
use crate::models::note::{CreateNoteRequest, UpdateNoteRequest, Note};
use axum::http::StatusCode;
use uuid::Uuid;

pub fn routes(pool: sqlx::SqlitePool) -> Router {
    Router::new()
        .route("/", post(create_note).get(list_notes))
        .route("/:id", put(update_note).delete(delete_note))
        .with_state(pool)
}

async fn create_note(
    State(pool): State<sqlx::SqlitePool>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<Json<Note>, AppError> {
    let team = access::channel_access(&pool, payload.channel_id, user_id).await?;
    if team != payload.team_id { return Err(AppError { status: StatusCode::BAD_REQUEST, message: "Channel does not belong to team".into() }); }
    let note_id = Uuid::new_v4();
    let now = chrono::Utc::now().naive_utc();

    sqlx::query("INSERT INTO notes (id, team_id, channel_id, title, content, color, position_x, position_y, author_id, is_pinned, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(note_id)
        .bind(payload.team_id)
        .bind(payload.channel_id)
        .bind(&payload.title)
        .bind(&payload.content)
        .bind(payload.color.clone().unwrap_or_else(|| "#fff59d".into()))
        .bind(payload.position_x.unwrap_or(0))
        .bind(payload.position_y.unwrap_or(0))
        .bind(user_id)
        .bind(false)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: format!("Failed to create note: {}", e) })?;

    Ok(Json(Note {
        id: note_id,
        team_id: payload.team_id,
        channel_id: payload.channel_id,
        title: payload.title,
        content: payload.content,
        color: payload.color.unwrap_or_else(|| "#fff59d".into()),
        position_x: payload.position_x.unwrap_or(0),
        position_y: payload.position_y.unwrap_or(0),
        author_id: user_id,
        is_pinned: false,
        created_at: now,
        updated_at: now,
    }))
}

async fn list_notes(
    State(pool): State<sqlx::SqlitePool>,
    Extension(user_id): Extension<Uuid>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<Note>>, AppError> {
    let channel_id = params["channel_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError { status: StatusCode::BAD_REQUEST, message: "channel_id required".into() })?;

    access::channel_access(&pool, channel_id, user_id).await?;
    let notes: Vec<Note> = sqlx::query_as("SELECT * FROM notes WHERE channel_id = ? ORDER BY is_pinned DESC, created_at DESC")
        .bind(channel_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: format!("Failed to list notes: {}", e) })?;

    Ok(Json(notes))
}

async fn update_note(
    State(pool): State<sqlx::SqlitePool>,
    Path(note_id): Path<Uuid>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<Note>, AppError> {
    ensure_note_access(&pool, note_id, user_id).await?;
    let now = chrono::Utc::now().naive_utc();

    sqlx::query("UPDATE notes SET title = COALESCE(?, title), content = COALESCE(?, content), color = COALESCE(?, color), position_x = COALESCE(?, position_x), position_y = COALESCE(?, position_y), is_pinned = COALESCE(?, is_pinned), updated_at = ? WHERE id = ?")
        .bind(payload.title)
        .bind(payload.content)
        .bind(payload.color)
        .bind(payload.position_x)
        .bind(payload.position_y)
        .bind(payload.is_pinned)
        .bind(now)
        .bind(note_id)
        .execute(&pool)
        .await
        .map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: format!("Failed to update note: {}", e) })?;

    let note: Option<Note> = sqlx::query_as("SELECT * FROM notes WHERE id = ?")
        .bind(note_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: format!("Failed to fetch note: {}", e) })?;

    Ok(Json(note.ok_or_else(|| AppError { status: StatusCode::NOT_FOUND, message: "Note not found".into() })?))
}

async fn delete_note(
    State(pool): State<sqlx::SqlitePool>,
    Path(note_id): Path<Uuid>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    ensure_note_access(&pool, note_id, user_id).await?;
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(&pool)
        .await
        .map_err(|e| AppError { status: StatusCode::INTERNAL_SERVER_ERROR, message: format!("Failed to delete note: {}", e) })?;

    Ok(Json(serde_json::json!({"status": "deleted"})))
}
async fn ensure_note_access(pool: &sqlx::SqlitePool, note: Uuid, user: Uuid) -> Result<(), AppError> {
    let channel: Option<Uuid> = sqlx::query_scalar("SELECT channel_id FROM notes WHERE id=?").bind(note).fetch_optional(pool).await?;
    let channel = channel.ok_or_else(|| AppError { status: StatusCode::NOT_FOUND, message: "Note not found".into() })?;
    access::channel_access(pool, channel, user).await?; Ok(())
}
