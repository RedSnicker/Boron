use axum::{extract::{State,Extension,Path,Multipart,DefaultBodyLimit},routing::post,Json,Router,http::StatusCode};
use crate::{storage,models::error::AppError};
use uuid::Uuid;
pub fn routes(pool:sqlx::SqlitePool)->Router {Router::new().route("/",post(upload).get(list).layer(DefaultBodyLimit::max(storage::MAX_REQUEST))).route("/:id",axum::routing::delete(delete)).with_state(pool)}
async fn list(State(pool):State<sqlx::SqlitePool>,Extension(_user):Extension<Uuid>)->Result<Json<Vec<serde_json::Value>>,AppError>{
 let rows=sqlx::query_as::<_,(Uuid,Uuid,String,String)>("SELECT id,uploader_id,filename,mime FROM stored_files WHERE scope='sticker' ORDER BY created_at DESC").fetch_all(&pool).await?;
 Ok(Json(rows.into_iter().map(|(id,owner_id,name,mime)|serde_json::json!({"id":id,"owner_id":owner_id,"name":name,"mime":mime})).collect()))
}
async fn upload(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,multipart:Multipart)->Result<Json<serde_json::Value>,AppError>{
 let file=storage::receive(multipart,true).await?;storage::store(&pool,"sticker",user,user,&file).await?;
 Ok(Json(serde_json::json!({"id":file.id,"owner_id":user,"name":file.filename,"mime":file.mime})))
}
async fn delete(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>,AppError>{
 let owner:Option<Uuid>=sqlx::query_scalar("SELECT uploader_id FROM stored_files WHERE id=? AND scope='sticker'").bind(id).fetch_optional(&pool).await?;
 let owner=owner.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"Sticker not found.".into()})?;
 let admin:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id=? AND role='admin' AND is_active=1)").bind(user).fetch_one(&pool).await?;
 if owner!=user&&!admin{return Err(AppError{status:StatusCode::FORBIDDEN,message:"Only the owner or an admin can delete this sticker.".into()});}
 sqlx::query("DELETE FROM stored_files WHERE id=? AND scope='sticker'").bind(id).execute(&pool).await?;
 let _=tokio::fs::remove_file(storage::path(id)).await;Ok(Json(serde_json::json!({"status":"deleted"})))
}
