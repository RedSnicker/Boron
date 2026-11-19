use axum::{routing::{get, post}, Json, Router, extract::{State, Extension, Path, Query, Multipart, DefaultBodyLimit}};
use crate::{access, models::{error::AppError, team::{CreateTeamRequest, TeamResponse}}};
use axum::http::StatusCode;
use uuid::Uuid;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
struct Member { id: Uuid, email: String, display_name: String, avatar_url: Option<String>, role: String }

pub fn routes(pool: sqlx::SqlitePool) -> Router {
    Router::new().route("/", post(create_team).get(list_teams))
        .route("/:id/avatar", post(upload_avatar).layer(DefaultBodyLimit::max(crate::storage::MAX_REQUEST)))
        .route("/:id/members", post(add_member).get(list_members)).route("/:id/user-search", get(search_users)).with_state(pool)
}
async fn list_teams(State(pool): State<sqlx::SqlitePool>, Extension(user): Extension<Uuid>) -> Result<Json<Vec<TeamResponse>>, AppError> {
    let teams = sqlx::query_as::<_, (Uuid, String, String, Option<String>, Uuid, i64, String, Option<Uuid>)>("SELECT t.id,t.name,t.slug,t.description,t.owner_id,(SELECT COUNT(*) FROM team_members WHERE team_id=t.id),tm.role,(SELECT sf.id FROM stored_files sf WHERE sf.scope='team_avatar' AND sf.parent_id=t.id ORDER BY sf.rowid DESC LIMIT 1) FROM teams t JOIN team_members tm ON tm.team_id=t.id WHERE tm.user_id=? ORDER BY t.name")
        .bind(user).fetch_all(&pool).await?.into_iter().map(|(id,name,slug,description,owner_id,member_count,role,avatar)| TeamResponse { id,name,slug,description,owner_id,member_count,role,avatar_url:avatar.map(|v|format!("/api/team-avatars/{}?v={}",id,v)) }).collect();
    Ok(Json(teams))
}
async fn create_team(State(pool): State<sqlx::SqlitePool>, Extension(user_id): Extension<Uuid>, Json(payload): Json<CreateTeamRequest>) -> Result<Json<TeamResponse>, AppError> {
    let name = payload.name.trim();
    let slug: String = payload.slug.trim().to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' {c} else {'-'}).collect::<String>().trim_matches('-').to_owned();
    if name.is_empty() || slug.is_empty() { return Err(AppError {status:StatusCode::BAD_REQUEST,message:"A team name and slug are required.".into()}); }
    let id=Uuid::new_v4(); let now=chrono::Utc::now().naive_utc();
    let mut tx=pool.begin().await?;
    let mut final_slug=slug.clone();
    let inserted = sqlx::query("INSERT INTO teams (id,name,slug,description,owner_id,created_at,updated_at) VALUES (?,?,?,?,?,?,?)").bind(id).bind(name).bind(&final_slug).bind(&payload.description).bind(user_id).bind(now).bind(now).execute(&mut *tx).await;
    if let Err(e)=inserted {
        if e.as_database_error().is_some_and(|d| d.is_unique_violation()) {
            final_slug=format!("{}-{}",slug,&id.to_string()[..8]);
            sqlx::query("INSERT INTO teams (id,name,slug,description,owner_id,created_at,updated_at) VALUES (?,?,?,?,?,?,?)").bind(id).bind(name).bind(&final_slug).bind(&payload.description).bind(user_id).bind(now).bind(now).execute(&mut *tx).await?;
        } else { return Err(e.into()); }
    }
    sqlx::query("INSERT INTO team_members (id,team_id,user_id,role,joined_at) VALUES (?,?,?,'owner',?)").bind(Uuid::new_v4()).bind(id).bind(user_id).bind(now).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(TeamResponse { id,name:name.into(),slug:final_slug,description:payload.description,owner_id:user_id,member_count:1,role:"owner".into(),avatar_url:None }))
}
async fn list_members(State(pool): State<sqlx::SqlitePool>, Extension(user): Extension<Uuid>, Path(team): Path<Uuid>) -> Result<Json<Vec<Member>>,AppError> {
    access::team_member(&pool,team,user).await?;
    let rows=sqlx::query_as::<_, Member>("SELECT u.id,u.email,u.display_name,u.avatar_url,tm.role FROM users u JOIN team_members tm ON u.id=tm.user_id WHERE tm.team_id=? ORDER BY u.display_name").bind(team).fetch_all(&pool).await?;
    Ok(Json(rows))
}
async fn add_member(State(pool): State<sqlx::SqlitePool>, Extension(user): Extension<Uuid>, Path(team): Path<Uuid>, Json(payload): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, AppError> {
    access::team_admin(&pool,team,user).await?;
    let member=resolve_user(&pool, &payload).await?;
    sqlx::query("INSERT OR IGNORE INTO team_members (id,team_id,user_id,role,joined_at) VALUES (?,?,?,'member',?)").bind(Uuid::new_v4()).bind(team).bind(member).bind(chrono::Utc::now().naive_utc()).execute(&pool).await?;
    Ok(Json(serde_json::json!({"id":member})))
}

#[derive(serde::Deserialize)] struct Search { q:String }
async fn search_users(State(pool):State<sqlx::SqlitePool>, Extension(user):Extension<Uuid>, Path(team):Path<Uuid>, Query(q):Query<Search>) -> Result<Json<Vec<serde_json::Value>>,AppError> {
 access::team_admin(&pool,team,user).await?;
 let term=q.q.trim(); if term.chars().count()<2{return Ok(Json(vec![]));}
 let pattern=format!("%{}%",term.replace('%',"\\%").replace('_',"\\_"));
 let rows=sqlx::query_as::<_,(Uuid,String,String,Option<String>)>("SELECT id,display_name,email,avatar_url FROM users WHERE is_active=1 AND (display_name LIKE ? ESCAPE '\\' OR email LIKE ? ESCAPE '\\') ORDER BY display_name LIMIT 10").bind(&pattern).bind(&pattern).fetch_all(&pool).await?;
 Ok(Json(rows.into_iter().map(|(id,display_name,email,avatar_url)|serde_json::json!({"id":id,"display_name":display_name,"email":email,"avatar_url":avatar_url})).collect()))
}
pub async fn resolve_user(pool:&sqlx::SqlitePool,payload:&serde_json::Value)->Result<Uuid,AppError> {
 if let Some(raw)=payload["user_id"].as_str(){let id=Uuid::parse_str(raw).map_err(|_|AppError{status:StatusCode::BAD_REQUEST,message:"Invalid user.".into()})?;let found:Option<Uuid>=sqlx::query_scalar("SELECT id FROM users WHERE id=? AND is_active=1").bind(id).fetch_optional(pool).await?;return found.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"User not found.".into()});}
 let term=payload["email"].as_str().or_else(||payload["username"].as_str()).unwrap_or("").trim();
 let matches:Vec<Uuid>=sqlx::query_scalar("SELECT id FROM users WHERE is_active=1 AND (email=? COLLATE NOCASE OR display_name=? COLLATE NOCASE) LIMIT 2").bind(term).bind(term).fetch_all(pool).await?;
 match matches.as_slice(){[id]=>Ok(*id),[]=>Err(AppError{status:StatusCode::NOT_FOUND,message:"No active account found.".into()}),_=>Err(AppError{status:StatusCode::CONFLICT,message:"Multiple people share that name. Select a suggestion.".into()})}
}

async fn upload_avatar(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(team):Path<Uuid>,multipart:Multipart)->Result<Json<serde_json::Value>,AppError>{
 access::team_admin(&pool,team,user).await?;let upload=crate::storage::receive(multipart,true).await?;crate::storage::store(&pool,"team_avatar",team,user,&upload).await?;
 let old:Vec<Uuid>=sqlx::query_scalar("SELECT id FROM stored_files WHERE scope='team_avatar' AND parent_id=? AND id!=?").bind(team).bind(upload.id).fetch_all(&pool).await?;
 for id in old {sqlx::query("DELETE FROM stored_files WHERE id=?").bind(id).execute(&pool).await?;let _=tokio::fs::remove_file(crate::storage::path(id)).await;}
 Ok(Json(serde_json::json!({"avatar_url":format!("/api/team-avatars/{}?v={}",team,upload.id)})))
}
