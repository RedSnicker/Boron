use axum::{routing::{get, post, put}, Json, Router, extract::{State, Extension, Path, Query}};
use crate::{access, models::{error::AppError, channel::{CreateChannelRequest, Channel}}};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)] struct ChannelQuery { team_id: Uuid }
#[derive(Deserialize)] struct Content { content: String }
pub fn routes(pool: sqlx::SqlitePool) -> Router {
    Router::new().route("/", post(create_channel).get(list_channels))
        .route("/unread", get(crate::routes::chat::unread))
        .route("/:id", axum::routing::delete(delete_channel))
        .route("/:id/reorder", put(reorder_channel))
        .route("/:id/members", post(add_member).get(list_members))
        .route("/:id/pins", get(crate::routes::chat::list_pins))
        .route("/:id/messages", get(crate::routes::chat::list_messages).post(crate::routes::chat::send_message))
        .route("/:id/messages/:message", put(crate::routes::chat::edit_message).delete(crate::routes::chat::delete_message))
        .route("/:id/messages/:message/pin", post(crate::routes::chat::pin_message))
        .route("/:id/messages/:message/reminders", post(crate::routes::chat::create_reminder))
        .route("/:id/messages/read", post(crate::routes::chat::mark_read))
        .route("/:id/messages/:message/reactions", post(crate::routes::chat::toggle_reaction))
        .route("/:id/messages/:message/attachments", post(crate::routes::chat::upload).layer(axum::extract::DefaultBodyLimit::max(crate::storage::MAX_REQUEST)))
        .route("/:id/messages/:message/attachments/:file", get(crate::routes::chat::download))
        .route("/:id/references", get(crate::routes::chat::references))
        .route("/:id/page", get(get_page).put(save_page)).with_state(pool)
}
async fn create_channel(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Json(p):Json<CreateChannelRequest>) -> Result<Json<Channel>,AppError> {
    access::team_admin(&pool,p.team_id,user).await?;
    let name=p.name.trim(); if name.is_empty() {return Err(AppError{status:StatusCode::BAD_REQUEST,message:"A name is required.".into()});}
    let id=Uuid::new_v4(); let now=chrono::Utc::now().naive_utc();
    let mut slug=p.slug.trim().to_lowercase(); if slug.is_empty() {slug=id.to_string();}
    let private=if p.r#type==crate::models::channel::ChannelType::Text || p.r#type==crate::models::channel::ChannelType::General {p.is_private.unwrap_or(true)} else {true};
    let position=p.position.unwrap_or(0);
    let icon=p.icon.clone();
    let mut tx=pool.begin().await?;
    let insert=sqlx::query("INSERT INTO channels (id,team_id,name,slug,type,description,is_private,required_role,position,icon,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,NULL,?,?)").bind(id).bind(p.team_id).bind(name).bind(&slug).bind(p.r#type.clone()).bind(&p.description).bind(private).bind(position).bind(icon).bind(now).bind(now).execute(&mut *tx).await;
    if let Err(e)=insert { if e.as_database_error().is_some_and(|d|d.is_unique_violation()) {slug=format!("{}-{}",slug,&id.to_string()[..8]); sqlx::query("INSERT INTO channels (id,team_id,name,slug,type,description,is_private,required_role,position,icon,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,NULL,?,?)").bind(id).bind(p.team_id).bind(name).bind(&slug).bind(p.r#type.clone()).bind(&p.description).bind(private).bind(position).bind(icon).bind(now).bind(now).execute(&mut *tx).await?;} else{return Err(e.into());}}
    sqlx::query("INSERT INTO channel_members (channel_id,user_id) VALUES (?,?)").bind(id).bind(user).execute(&mut *tx).await?; tx.commit().await?;
    Ok(Json(Channel{id,team_id:p.team_id,name:name.into(),slug,r#type:p.r#type,description:p.description,is_private:private,required_role:None,position,icon,created_at:now,updated_at:now}))
}
async fn list_channels(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Query(q):Query<ChannelQuery>) -> Result<Json<Vec<Channel>>,AppError> {
    access::team_member(&pool,q.team_id,user).await?;
    let channels=sqlx::query_as::<_,Channel>("SELECT c.* FROM channels c JOIN team_members tm ON tm.team_id=c.team_id AND tm.user_id=? WHERE c.team_id=? AND (c.is_private=0 OR tm.role IN ('owner','admin') OR EXISTS(SELECT 1 FROM channel_members cm WHERE cm.channel_id=c.id AND cm.user_id=?)) ORDER BY c.position ASC, c.name ASC").bind(user).bind(q.team_id).bind(user).fetch_all(&pool).await?;
    Ok(Json(channels))
}

#[derive(Deserialize)] struct ReorderChannel { channel_id: Uuid, position: i32 }
async fn reorder_channel(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Json(p):Json<ReorderChannel>) -> Result<Json<serde_json::Value>,AppError> {
    let team:Option<Uuid>=sqlx::query_scalar("SELECT team_id FROM channels WHERE id=?").bind(p.channel_id).fetch_optional(&pool).await?;
    let team=team.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"Channel not found".into()})?;
    access::team_admin(&pool,team,user).await?;
    sqlx::query("UPDATE channels SET position=?, updated_at=? WHERE id=?").bind(p.position).bind(chrono::Utc::now().naive_utc()).bind(p.channel_id).execute(&pool).await?;
    Ok(Json(serde_json::json!({"status":"reordered"})))
}
async fn list_members(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Path(channel):Path<Uuid>) -> Result<Json<Vec<serde_json::Value>>,AppError> {
    let team=access::channel_access(&pool,channel,user).await?;
    let rows=sqlx::query_as::<_,(Uuid,String,Option<String>)>("SELECT u.id,u.display_name,u.avatar_url FROM users u JOIN team_members tm ON tm.user_id=u.id WHERE tm.team_id=? AND (tm.role IN ('owner','admin') OR EXISTS(SELECT 1 FROM channel_members cm WHERE cm.channel_id=? AND cm.user_id=u.id) OR EXISTS(SELECT 1 FROM channels c WHERE c.id=? AND c.is_private=0)) ORDER BY u.display_name").bind(team).bind(channel).bind(channel).fetch_all(&pool).await?;
    Ok(Json(rows.into_iter().map(|(id,display_name,avatar_url)|serde_json::json!({"id":id,"display_name":display_name,"avatar_url":avatar_url})).collect()))
}
async fn add_member(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Path(channel):Path<Uuid>, Json(p):Json<serde_json::Value>) -> Result<Json<serde_json::Value>,AppError> {
    let team:Option<Uuid>=sqlx::query_scalar("SELECT team_id FROM channels WHERE id=?").bind(channel).fetch_optional(&pool).await?;
    let team=team.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"Channel not found".into()})?; access::team_admin(&pool,team,user).await?;
    let member=crate::routes::teams::resolve_user(&pool,&p).await?;
    let mut tx=pool.begin().await?;
    sqlx::query("INSERT OR IGNORE INTO team_members (id,team_id,user_id,role,joined_at) VALUES (?,?,?,'member',?)").bind(Uuid::new_v4()).bind(team).bind(member).bind(chrono::Utc::now().naive_utc()).execute(&mut *tx).await?;
    sqlx::query("INSERT OR IGNORE INTO channel_members (channel_id,user_id) VALUES (?,?)").bind(channel).bind(member).execute(&mut *tx).await?; tx.commit().await?;
    Ok(Json(serde_json::json!({"status":"ok"})))
}
async fn get_page(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Path(channel):Path<Uuid>) -> Result<Json<serde_json::Value>,AppError> {
    access::channel_access(&pool,channel,user).await?;
    let content:Option<String>=sqlx::query_scalar("SELECT content FROM channel_pages WHERE channel_id=?").bind(channel).fetch_optional(&pool).await?;
    Ok(Json(serde_json::json!({"content":content.unwrap_or_default()})))
}
async fn save_page(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Path(channel):Path<Uuid>, Json(p):Json<Content>) -> Result<Json<serde_json::Value>,AppError> {
    access::channel_access(&pool,channel,user).await?;
    sqlx::query("INSERT INTO channel_pages (channel_id,content,updated_at) VALUES (?,?,?) ON CONFLICT(channel_id) DO UPDATE SET content=excluded.content,updated_at=excluded.updated_at").bind(channel).bind(&p.content).bind(chrono::Utc::now().naive_utc()).execute(&pool).await?;
    Ok(Json(serde_json::json!({"status":"saved"})))
}

async fn delete_channel(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(channel):Path<Uuid>)->Result<Json<serde_json::Value>,AppError>{
 let team:Option<Uuid>=sqlx::query_scalar("SELECT team_id FROM channels WHERE id=?").bind(channel).fetch_optional(&pool).await?;
 let team=team.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"Channel not found.".into()})?;access::team_admin(&pool,team,user).await?;
 let mut tx=pool.begin().await?;
 let ids:Vec<Uuid>=sqlx::query_scalar("SELECT id FROM stored_files WHERE (scope='message' AND parent_id IN (SELECT id FROM messages WHERE channel_id=?)) OR (scope='issue' AND parent_id IN (SELECT id FROM issues WHERE channel_id=?))").bind(channel).bind(channel).fetch_all(&mut *tx).await?;
 for id in &ids {sqlx::query("DELETE FROM stored_files WHERE id=?").bind(id).execute(&mut *tx).await?;}
 sqlx::query("DELETE FROM channels WHERE id=?").bind(channel).execute(&mut *tx).await?;tx.commit().await?;
 for id in ids {let _=tokio::fs::remove_file(crate::storage::path(id)).await;}
 Ok(Json(serde_json::json!({"status":"deleted"})))
}
