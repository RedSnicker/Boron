use axum::{routing::{post, put, get}, Json, Router, extract::{State, Extension, Path, Query, Multipart, DefaultBodyLimit}, response::Response};
use crate::{access, models::{error::AppError, issue::{CreateIssueRequest, Issue}}};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;
#[derive(Deserialize)] struct IssueQuery {channel_id:Uuid}
#[derive(Deserialize)] struct Attachment { filename:String, data_url:String }
pub fn routes(pool:sqlx::SqlitePool)->Router {
 Router::new().route("/",post(create_issue).get(list_issues)).route("/:id",put(update_issue).delete(delete_issue)).route("/:id/attachments",post(add_attachment).get(list_attachments)).route("/:id/attachments/upload",post(upload_attachment).layer(DefaultBodyLimit::max(crate::storage::MAX_REQUEST))).route("/:id/attachments/:attachment/file",get(download_attachment)).route("/:id/attachments/:attachment",get(get_attachment)).with_state(pool)
}
fn bad(msg:&str)->AppError {AppError{status:StatusCode::BAD_REQUEST,message:msg.into()}}
async fn issue_channel(pool:&sqlx::SqlitePool,id:Uuid,user:Uuid)->Result<Uuid,AppError> {
 let channel:Option<Uuid>=sqlx::query_scalar("SELECT channel_id FROM issues WHERE id=?").bind(id).fetch_optional(pool).await?;
 let channel=channel.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"Issue not found".into()})?; access::channel_access(pool,channel,user).await?; Ok(channel)
}
async fn check_assignees(pool:&sqlx::SqlitePool,channel:Uuid,assignees:&[Uuid])->Result<(),AppError>{
 if assignees.len()>100{return Err(bad("An issue can have at most 100 assignees."));}
 for id in assignees {access::channel_access(pool,channel,*id).await.map_err(|_|bad("Each assignee must have access to this channel."))?;}Ok(())
}
fn unique_assignees(ids:Vec<Uuid>)->Vec<Uuid>{let mut unique=Vec::new();for id in ids {if !unique.contains(&id){unique.push(id);}}unique}
async fn issue_value(pool:&sqlx::SqlitePool,issue:Issue,include_attachments:bool)->Result<serde_json::Value,AppError>{
 let ids:Vec<Uuid>=sqlx::query_scalar("SELECT user_id FROM issue_assignees WHERE issue_id=? ORDER BY position,user_id").bind(issue.id).fetch_all(pool).await?;
 let attachments=if include_attachments {Some(attachments_for(pool,issue.id).await?)}else{None};
 let mut value=serde_json::to_value(issue).map_err(|_|bad("Could not serialize issue."))?;value["assignee_ids"]=serde_json::json!(ids);if let Some(files)=attachments{value["attachments"]=serde_json::json!(files);}Ok(value)
}
async fn create_issue(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Json(p):Json<CreateIssueRequest>)->Result<Json<serde_json::Value>,AppError>{
 let team=access::channel_access(&pool,p.channel_id,user).await?;
 if team!=p.team_id{return Err(bad("Channel does not belong to that team."));} if p.title.trim().is_empty(){return Err(bad("An issue title is required."));}
 let assignees=unique_assignees(p.assignee_ids.unwrap_or_else(||p.assignee_id.into_iter().collect()));check_assignees(&pool,p.channel_id,&assignees).await?;
 let id=Uuid::new_v4();let now=chrono::Utc::now().naive_utc();let priority=p.priority.unwrap_or(crate::models::issue::IssuePriority::Medium);
 let mut tx=pool.begin().await?;
 sqlx::query("INSERT INTO issues (id,team_id,channel_id,title,description,status,priority,assignee_id,reporter_id,created_at,updated_at) VALUES (?,?,?,?,?,'open',?,?,?,?,?)").bind(id).bind(team).bind(p.channel_id).bind(p.title.trim()).bind(&p.description).bind(priority).bind(assignees.first().copied()).bind(user).bind(now).bind(now).execute(&mut *tx).await?;
 for (position,assignee) in assignees.iter().enumerate(){sqlx::query("INSERT INTO issue_assignees (issue_id,user_id,position) VALUES (?,?,?)").bind(id).bind(assignee).bind(position as i64).execute(&mut *tx).await?;}tx.commit().await?;
 let issue=sqlx::query_as::<_,Issue>("SELECT * FROM issues WHERE id=?").bind(id).fetch_one(&pool).await?; Ok(Json(issue_value(&pool,issue,false).await?))
}
async fn list_issues(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Query(q):Query<IssueQuery>)->Result<Json<Vec<serde_json::Value>>,AppError>{
 access::channel_access(&pool,q.channel_id,user).await?;
 let rows=sqlx::query_as::<_,Issue>("SELECT * FROM issues WHERE channel_id=? ORDER BY created_at DESC").bind(q.channel_id).fetch_all(&pool).await?;
 let mut result=Vec::new();for issue in rows {result.push(issue_value(&pool,issue,true).await?);}Ok(Json(result))
}
async fn update_issue(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(id):Path<Uuid>,Json(p):Json<serde_json::Value>)->Result<Json<serde_json::Value>,AppError>{
 let channel=issue_channel(&pool,id,user).await?;
 let issue=sqlx::query_as::<_,Issue>("SELECT * FROM issues WHERE id=?").bind(id).fetch_one(&pool).await?;
 let status=p["status"].as_str().map(str::to_owned).unwrap_or_else(||serde_json::to_value(&issue.status).unwrap().as_str().unwrap().to_owned());
 if !["open","in_progress","review","done","closed"].contains(&status.as_str()){return Err(bad("Invalid status."));}
 let priority=p["priority"].as_str().map(str::to_owned).unwrap_or_else(||serde_json::to_value(&issue.priority).unwrap().as_str().unwrap().to_owned());
 if !["low","medium","high","critical"].contains(&priority.as_str()){return Err(bad("Invalid priority."));}
 let new_assignees=if let Some(value)=p.get("assignee_ids") {let raw=value.as_array().ok_or_else(||bad("Assignees must be a list."))?;if raw.len()>100{return Err(bad("An issue can have at most 100 assignees."));}let mut ids=Vec::new();for value in raw {ids.push(Uuid::parse_str(value.as_str().ok_or_else(||bad("Invalid assignee."))?).map_err(|_|bad("Invalid assignee."))?);}Some(unique_assignees(ids))}
 else if let Some(value)=p.get("assignee_id") {Some(if value.is_null(){Vec::new()}else{vec![Uuid::parse_str(value.as_str().ok_or_else(||bad("Invalid assignee."))?).map_err(|_|bad("Invalid assignee."))?]})} else {None};
 if let Some(ids)=&new_assignees {check_assignees(&pool,channel,ids).await?;}
 let primary=new_assignees.as_ref().map(|ids|ids.first().copied()).unwrap_or(issue.assignee_id);
 let title=p["title"].as_str().unwrap_or(&issue.title).trim(); if title.is_empty(){return Err(bad("An issue title is required."));}
 let description=if p.get("description").is_some(){p["description"].as_str().map(str::to_owned)} else {issue.description};
 let mut tx=pool.begin().await?;
 sqlx::query("UPDATE issues SET title=?,description=?,status=?,priority=?,assignee_id=?,updated_at=? WHERE id=?").bind(title).bind(description).bind(status).bind(priority).bind(primary).bind(chrono::Utc::now().naive_utc()).bind(id).execute(&mut *tx).await?;
 if let Some(ids)=new_assignees {sqlx::query("DELETE FROM issue_assignees WHERE issue_id=?").bind(id).execute(&mut *tx).await?;for (position,assignee) in ids.iter().enumerate(){sqlx::query("INSERT INTO issue_assignees (issue_id,user_id,position) VALUES (?,?,?)").bind(id).bind(assignee).bind(position as i64).execute(&mut *tx).await?;}}tx.commit().await?;
 let updated=sqlx::query_as::<_,Issue>("SELECT * FROM issues WHERE id=?").bind(id).fetch_one(&pool).await?;Ok(Json(issue_value(&pool,updated,false).await?))
}
async fn add_attachment(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(id):Path<Uuid>,Json(p):Json<Attachment>)->Result<Json<serde_json::Value>,AppError>{
 issue_channel(&pool,id,user).await?; if p.filename.trim().is_empty()||p.filename.len()>255||p.data_url.len()>1_400_000||!p.data_url.starts_with("data:")||!p.data_url.contains(";base64,"){return Err(bad("Attachment must be under 1 MB."));}
 let attachment=Uuid::new_v4(); sqlx::query("INSERT INTO issue_attachments (id,issue_id,filename,data_url) VALUES (?,?,?,?)").bind(attachment).bind(id).bind(&p.filename).bind(&p.data_url).execute(&pool).await?;
 Ok(Json(serde_json::json!({"id":attachment,"filename":p.filename})))
}
async fn attachments_for(pool:&sqlx::SqlitePool,id:Uuid)->Result<Vec<serde_json::Value>,AppError>{
 let old=sqlx::query_as::<_,(Uuid,String)>("SELECT id,filename FROM issue_attachments WHERE issue_id=? ORDER BY created_at").bind(id).fetch_all(pool).await?;
 let new=sqlx::query_as::<_,(Uuid,String,String,i64)>("SELECT id,filename,mime,size FROM stored_files WHERE scope='issue' AND parent_id=? ORDER BY created_at").bind(id).fetch_all(pool).await?;
 Ok(old.into_iter().map(|(id,filename)|serde_json::json!({"id":id,"filename":filename,"legacy":true})).chain(new.into_iter().map(|(id,filename,mime,size)|serde_json::json!({"id":id,"filename":filename,"mime":mime,"size":size,"legacy":false}))).collect())
}
async fn list_attachments(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(id):Path<Uuid>)->Result<Json<Vec<serde_json::Value>>,AppError>{
 issue_channel(&pool,id,user).await?; Ok(Json(attachments_for(&pool,id).await?))
}
async fn upload_attachment(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(id):Path<Uuid>,multipart:Multipart)->Result<Json<serde_json::Value>,AppError>{
 issue_channel(&pool,id,user).await?;let file=crate::storage::receive(multipart,false).await?;crate::storage::store(&pool,"issue",id,user,&file).await?;
 Ok(Json(serde_json::json!({"id":file.id,"filename":file.filename,"mime":file.mime,"size":file.size,"legacy":false})))
}
async fn download_attachment(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((issue,id)):Path<(Uuid,Uuid)>)->Result<Response,AppError>{
 issue_channel(&pool,issue,user).await?;let mime:Option<String>=sqlx::query_scalar("SELECT mime FROM stored_files WHERE id=? AND scope='issue' AND parent_id=?").bind(id).bind(issue).fetch_optional(&pool).await?;
 crate::storage::serve(id,&mime.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"File not found".into()})?).await
}
async fn get_attachment(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((issue,id)):Path<(Uuid,Uuid)>)->Result<Json<serde_json::Value>,AppError>{
 issue_channel(&pool,issue,user).await?; let row:Option<(String,String)>=sqlx::query_as("SELECT filename,data_url FROM issue_attachments WHERE id=? AND issue_id=?").bind(id).bind(issue).fetch_optional(&pool).await?;
 let (filename,data_url)=row.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"Attachment not found".into()})?; Ok(Json(serde_json::json!({"filename":filename,"data_url":data_url})))
}

async fn delete_issue(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>,AppError>{
 issue_channel(&pool,id,user).await?;let mut tx=pool.begin().await?;
 let files:Vec<Uuid>=sqlx::query_scalar("SELECT id FROM stored_files WHERE scope='issue' AND parent_id=?").bind(id).fetch_all(&mut *tx).await?;
 sqlx::query("DELETE FROM stored_files WHERE scope='issue' AND parent_id=?").bind(id).execute(&mut *tx).await?;
 sqlx::query("DELETE FROM issues WHERE id=?").bind(id).execute(&mut *tx).await?;tx.commit().await?;
 for file in files {let _=tokio::fs::remove_file(crate::storage::path(file)).await;}
 Ok(Json(serde_json::json!({"status":"deleted"})))
}
