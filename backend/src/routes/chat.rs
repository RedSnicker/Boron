use axum::{extract::{State,Extension,Path,Query,Multipart},Json,response::Response,http::StatusCode};
use crate::{access,models::error::AppError,storage};
use serde::Deserialize;
use uuid::Uuid;

fn bad(msg:&str)->AppError{AppError{status:StatusCode::BAD_REQUEST,message:msg.into()}}
#[derive(Deserialize)] pub struct NewMessage {content:String,parent_id:Option<Uuid>,message_type:Option<String>,has_attachments:Option<bool>}
#[derive(Deserialize)] pub struct Reaction {emoji:String}
#[derive(Deserialize)] pub struct Search {q:String}
async fn check_message(pool:&sqlx::SqlitePool,channel:Uuid,message:Uuid,user:Uuid)->Result<Uuid,AppError>{
 access::channel_access(pool,channel,user).await?;
 let author:Option<Uuid>=sqlx::query_scalar("SELECT user_id FROM messages WHERE id=? AND channel_id=?").bind(message).bind(channel).fetch_optional(pool).await?;
 author.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"Message not found.".into()})
}
pub async fn list_messages(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(channel):Path<Uuid>)->Result<Json<Vec<serde_json::Value>>,AppError>{
 access::channel_access(&pool,channel,user).await?;
 let mut rows=sqlx::query_as::<_,(Uuid,Uuid,String,String,String,Option<String>,String,Option<Uuid>,Option<String>,bool)>("SELECT m.id,m.user_id,m.content,m.created_at,u.display_name,u.avatar_url,m.message_type,m.parent_id,m.edited_at,m.is_pinned FROM messages m JOIN users u ON u.id=m.user_id WHERE m.channel_id=? ORDER BY m.created_at DESC,m.rowid DESC LIMIT 200").bind(channel).fetch_all(&pool).await?;rows.reverse();
 let mut results=Vec::new();for (id,user_id,content,created_at,display_name,avatar_url,message_type,parent_id,edited_at,is_pinned) in rows {
  let files=sqlx::query_as::<_,(Uuid,String,String,i64)>("SELECT id,filename,mime,size FROM stored_files WHERE scope='message' AND parent_id=? ORDER BY created_at").bind(id).fetch_all(&pool).await?;
  let reactions=sqlx::query_as::<_,(String,i64,i64)>("SELECT emoji,COUNT(*),MAX(CASE WHEN user_id=? THEN 1 ELSE 0 END) FROM message_reactions WHERE message_id=? GROUP BY emoji ORDER BY emoji").bind(user).bind(id).fetch_all(&pool).await?;
  let is_mentioned:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM message_mentions WHERE message_id=? AND user_id=?)").bind(id).bind(user).fetch_one(&pool).await?;
  results.push(serde_json::json!({"is_mentioned":is_mentioned,"edited_at":edited_at,"is_pinned":is_pinned,"id":id,"user_id":user_id,"content":content,"created_at":created_at,"display_name":display_name,"avatar_url":avatar_url,"message_type":message_type,"parent_id":parent_id,
    "attachments":files.into_iter().map(|(id,filename,mime,size)|serde_json::json!({"id":id,"filename":filename,"mime":mime,"size":size})).collect::<Vec<_>>(),
    "reactions":reactions.into_iter().map(|(emoji,count,mine)|serde_json::json!({"emoji":emoji,"count":count,"mine":mine==1})).collect::<Vec<_>>() }));
 } Ok(Json(results))
}
pub async fn send_message(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(channel):Path<Uuid>,Json(p):Json<NewMessage>)->Result<Json<serde_json::Value>,AppError>{
 access::channel_access(&pool,channel,user).await?;if p.content.len()>100_000{return Err(bad("Message is too long."));}
 if p.content.trim().is_empty()&&!p.has_attachments.unwrap_or(false){return Err(bad("Message is empty."));}
 if let Some(parent)=p.parent_id {check_message(&pool,channel,parent,user).await?;}
 let kind=p.message_type.as_deref().unwrap_or("text");if !["text","sticker"].contains(&kind) {return Err(bad("Invalid message type."));}
 if kind=="sticker" && p.content.len()>64{return Err(bad("Invalid sticker."));}
 if kind=="sticker" && p.content.starts_with("sticker:") {
   let sticker=Uuid::parse_str(&p.content[8..]).map_err(|_|bad("Invalid sticker."))?;
   let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM stored_files WHERE id=? AND scope='sticker')").bind(sticker).fetch_one(&pool).await?;
   if !exists {return Err(bad("Sticker no longer exists."));}
 }
 let mut mentions=Vec::new();
 if kind=="text" {for part in p.content.split("](boron-user:").skip(1).take(100) {if let Some(raw)=part.split(')').next(){if let Ok(id)=Uuid::parse_str(raw){if id!=user && !mentions.contains(&id) && access::channel_access(&pool,channel,id).await.is_ok(){mentions.push(id);}}}}}

 let id=Uuid::new_v4();let now=chrono::Utc::now().naive_utc();
 let mut tx=pool.begin().await?;
 sqlx::query("INSERT INTO messages (id,channel_id,user_id,content,message_type,parent_id,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?)").bind(id).bind(channel).bind(user).bind(&p.content).bind(kind).bind(p.parent_id).bind(now).bind(now).execute(&mut *tx).await?;
 for mentioned in mentions {sqlx::query("INSERT OR IGNORE INTO message_mentions (message_id,user_id) VALUES (?,?)").bind(id).bind(mentioned).execute(&mut *tx).await?;} tx.commit().await?;
 Ok(Json(serde_json::json!({"id":id})))
}
pub async fn toggle_reaction(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((channel,message)):Path<(Uuid,Uuid)>,Json(p):Json<Reaction>)->Result<Json<serde_json::Value>,AppError>{
 check_message(&pool,channel,message,user).await?;if p.emoji.trim().is_empty()||p.emoji.len()>40||p.emoji.chars().any(char::is_control){return Err(bad("Invalid reaction."));}
 let res=sqlx::query("DELETE FROM message_reactions WHERE message_id=? AND user_id=? AND emoji=?").bind(message).bind(user).bind(&p.emoji).execute(&pool).await?;
 if res.rows_affected()==0{sqlx::query("INSERT OR IGNORE INTO message_reactions (message_id,user_id,emoji) VALUES (?,?,?)").bind(message).bind(user).bind(&p.emoji).execute(&pool).await?;}
 Ok(Json(serde_json::json!({"status":"ok"})))
}
pub async fn upload(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((channel,message)):Path<(Uuid,Uuid)>,multipart:Multipart)->Result<Json<serde_json::Value>,AppError>{
 if check_message(&pool,channel,message,user).await?!=user{return Err(AppError{status:StatusCode::FORBIDDEN,message:"Only the sender may add files.".into()});}
 let file=storage::receive(multipart,false).await?; storage::store(&pool,"message",message,user,&file).await?;
 Ok(Json(serde_json::json!({"id":file.id,"filename":file.filename,"mime":file.mime,"size":file.size})))
}
pub async fn download(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((channel,message,file)):Path<(Uuid,Uuid,Uuid)>)->Result<Response,AppError>{
 check_message(&pool,channel,message,user).await?;let mime:Option<String>=sqlx::query_scalar("SELECT mime FROM stored_files WHERE id=? AND parent_id=? AND scope='message'").bind(file).bind(message).fetch_optional(&pool).await?;
 storage::serve(file,&mime.ok_or_else(||AppError{status:StatusCode::NOT_FOUND,message:"File not found.".into()})?).await
}
pub async fn references(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(channel):Path<Uuid>,Query(q):Query<Search>)->Result<Json<Vec<serde_json::Value>>,AppError>{
 access::channel_access(&pool,channel,user).await?;let term=q.q.trim();if term.is_empty(){return Ok(Json(vec![]));}
 let pattern=format!("%{}%",term.replace('%',"\\%").replace('_',"\\_"));
 let issues=sqlx::query_as::<_,(Uuid,Uuid,Uuid,String)>("SELECT i.id,i.team_id,i.channel_id,i.title FROM issues i JOIN channels c ON c.id=i.channel_id JOIN team_members tm ON tm.team_id=c.team_id AND tm.user_id=? WHERE (c.is_private=0 OR tm.role IN ('owner','admin') OR EXISTS(SELECT 1 FROM channel_members cm WHERE cm.channel_id=c.id AND cm.user_id=?)) AND (i.title LIKE ? ESCAPE '\\' OR lower(hex(i.id)) LIKE ?) ORDER BY i.updated_at DESC LIMIT 10").bind(user).bind(user).bind(&pattern).bind(format!("%{}%",term.replace('-', "").to_lowercase())).fetch_all(&pool).await?;
 Ok(Json(issues.into_iter().map(|(id,team_id,channel_id,title)|serde_json::json!({"id":id,"team_id":team_id,"channel_id":channel_id,"title":title})).collect()))
}

pub async fn unread(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>)->Result<Json<Vec<serde_json::Value>>,AppError>{
 let rows=sqlx::query_as::<_,(Uuid,Uuid,i64)>("SELECT c.id,c.team_id,COUNT(*) FROM message_mentions mm JOIN messages m ON m.id=mm.message_id JOIN channels c ON c.id=m.channel_id JOIN team_members tm ON tm.team_id=c.team_id AND tm.user_id=? WHERE mm.user_id=? AND mm.read_at IS NULL AND (c.is_private=0 OR tm.role IN ('owner','admin') OR EXISTS (SELECT 1 FROM channel_members cm WHERE cm.channel_id=c.id AND cm.user_id=?)) GROUP BY c.id,c.team_id").bind(user).bind(user).bind(user).fetch_all(&pool).await?;
 Ok(Json(rows.into_iter().map(|(channel_id,team_id,count)|serde_json::json!({"channel_id":channel_id,"team_id":team_id,"count":count})).collect()))
}
#[derive(Deserialize)] pub struct ReadUpTo {last_seen_id:Uuid}
pub async fn mark_read(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(channel):Path<Uuid>,Json(p):Json<ReadUpTo>)->Result<Json<serde_json::Value>,AppError>{
 access::channel_access(&pool,channel,user).await?;
 let seen:Option<String>=sqlx::query_scalar("SELECT created_at FROM messages WHERE id=? AND channel_id=?").bind(p.last_seen_id).bind(channel).fetch_optional(&pool).await?;
 let seen=seen.ok_or_else(||bad("Message not found in this channel."))?;
 sqlx::query("UPDATE message_mentions SET read_at=? WHERE user_id=? AND read_at IS NULL AND message_id IN (SELECT id FROM messages WHERE channel_id=? AND created_at<=?)").bind(chrono::Utc::now().naive_utc()).bind(user).bind(channel).bind(seen).execute(&pool).await?;
 Ok(Json(serde_json::json!({"status":"read"})))
}

#[derive(Deserialize)] pub struct EditMessage {content:String}
pub async fn edit_message(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((channel,id)):Path<(Uuid,Uuid)>,Json(p):Json<EditMessage>)->Result<Json<serde_json::Value>,AppError>{
 if check_message(&pool,channel,id,user).await?!=user {return Err(AppError{status:StatusCode::FORBIDDEN,message:"Only the sender can edit this message.".into()});}
 let kind:String=sqlx::query_scalar("SELECT message_type FROM messages WHERE id=?").bind(id).fetch_one(&pool).await?;
 if kind!="text" || p.content.trim().is_empty() || p.content.len()>100_000 {return Err(bad("Enter valid message text."));}
 let mut mentions=Vec::new();for part in p.content.split("](boron-user:").skip(1).take(100){if let Some(raw)=part.split(')').next(){if let Ok(id)=Uuid::parse_str(raw){if id!=user&&!mentions.contains(&id)&&access::channel_access(&pool,channel,id).await.is_ok(){mentions.push(id);}}}}
 let mut tx=pool.begin().await?;
 sqlx::query("UPDATE messages SET content=?,edited_at=?,updated_at=? WHERE id=?").bind(&p.content).bind(chrono::Utc::now().naive_utc()).bind(chrono::Utc::now().naive_utc()).bind(id).execute(&mut *tx).await?;
 let old:Vec<Uuid>=sqlx::query_scalar("SELECT user_id FROM message_mentions WHERE message_id=?").bind(id).fetch_all(&mut *tx).await?;
 for member in old {if !mentions.contains(&member){sqlx::query("DELETE FROM message_mentions WHERE message_id=? AND user_id=?").bind(id).bind(member).execute(&mut *tx).await?;}}
 for member in mentions {sqlx::query("INSERT OR IGNORE INTO message_mentions(message_id,user_id) VALUES (?,?)").bind(id).bind(member).execute(&mut *tx).await?;}tx.commit().await?;
 Ok(Json(serde_json::json!({"status":"updated"})))
}
pub async fn delete_message(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((channel,id)):Path<(Uuid,Uuid)>)->Result<Json<serde_json::Value>,AppError>{
 let author=check_message(&pool,channel,id,user).await?;if author!=user {let team=access::channel_access(&pool,channel,user).await?;access::team_admin(&pool,team,user).await?;}
 let mut tx=pool.begin().await?;let files:Vec<Uuid>=sqlx::query_scalar("SELECT id FROM stored_files WHERE scope='message' AND parent_id=?").bind(id).fetch_all(&mut *tx).await?;
 sqlx::query("DELETE FROM stored_files WHERE scope='message' AND parent_id=?").bind(id).execute(&mut *tx).await?;sqlx::query("DELETE FROM messages WHERE id=?").bind(id).execute(&mut *tx).await?;tx.commit().await?;
 for file in files {let _=tokio::fs::remove_file(storage::path(file)).await;}Ok(Json(serde_json::json!({"status":"deleted"})))
}
#[derive(Deserialize)] pub struct PinMessage {pinned:bool}
pub async fn pin_message(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((channel,id)):Path<(Uuid,Uuid)>,Json(p):Json<PinMessage>)->Result<Json<serde_json::Value>,AppError>{
 check_message(&pool,channel,id,user).await?;sqlx::query("UPDATE messages SET is_pinned=? WHERE id=?").bind(p.pinned).bind(id).execute(&pool).await?;Ok(Json(serde_json::json!({"status":"ok"})))
}
#[derive(Deserialize)] pub struct ReminderInput {remind_at:String}
pub async fn create_reminder(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path((channel,id)):Path<(Uuid,Uuid)>,Json(p):Json<ReminderInput>)->Result<Json<serde_json::Value>,AppError>{
 check_message(&pool,channel,id,user).await?;let at=chrono::DateTime::parse_from_rfc3339(&p.remind_at).map_err(|_|bad("Choose a valid reminder time."))?.with_timezone(&chrono::Utc);if at<=chrono::Utc::now(){return Err(bad("Choose a future reminder time."));}
 let rid=Uuid::new_v4();sqlx::query("INSERT INTO message_reminders(id,user_id,message_id,remind_at) VALUES (?,?,?,?)").bind(rid).bind(user).bind(id).bind(at.to_rfc3339()).execute(&pool).await?;
 Ok(Json(serde_json::json!({"id":rid,"remind_at":at.to_rfc3339()})))
}
pub async fn list_reminders(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>)->Result<Json<Vec<serde_json::Value>>,AppError>{
 let rows=sqlx::query_as::<_,(Uuid,Uuid,Uuid,Uuid,String,String,String)>("SELECT r.id,m.id,c.id,c.team_id,r.remind_at,m.content,c.name FROM message_reminders r JOIN messages m ON m.id=r.message_id JOIN channels c ON c.id=m.channel_id JOIN team_members tm ON tm.team_id=c.team_id AND tm.user_id=? WHERE r.user_id=? AND r.delivered_at IS NULL AND (c.is_private=0 OR tm.role IN ('owner','admin') OR EXISTS(SELECT 1 FROM channel_members cm WHERE cm.channel_id=c.id AND cm.user_id=?)) ORDER BY r.remind_at").bind(user).bind(user).bind(user).fetch_all(&pool).await?;
 Ok(Json(rows.into_iter().map(|(id,message_id,channel_id,team_id,remind_at,content,channel_name)|serde_json::json!({"id":id,"message_id":message_id,"channel_id":channel_id,"team_id":team_id,"remind_at":remind_at,"content":content,"channel_name":channel_name})).collect()))
}
pub async fn deliver_reminder(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(id):Path<Uuid>)->Result<Json<serde_json::Value>,AppError>{
 let changed=sqlx::query("UPDATE message_reminders SET delivered_at=? WHERE id=? AND user_id=? AND delivered_at IS NULL AND julianday(remind_at)<=julianday(?)").bind(chrono::Utc::now().to_rfc3339()).bind(id).bind(user).bind(chrono::Utc::now().to_rfc3339()).execute(&pool).await?;
 Ok(Json(serde_json::json!({"delivered":changed.rows_affected()==1})))
}

pub async fn list_pins(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(channel):Path<Uuid>)->Result<Json<Vec<serde_json::Value>>,AppError>{
 access::channel_access(&pool,channel,user).await?;
 let rows=sqlx::query_as::<_,(Uuid,Uuid,String,String,String,Option<String>,String,Option<String>)>("SELECT m.id,m.user_id,m.content,m.created_at,u.display_name,u.avatar_url,m.message_type,m.edited_at FROM messages m JOIN users u ON u.id=m.user_id WHERE m.channel_id=? AND m.is_pinned=1 ORDER BY m.created_at DESC,m.rowid DESC").bind(channel).fetch_all(&pool).await?;
 let mut results=Vec::new();for (id,user_id,content,created_at,display_name,avatar_url,message_type,edited_at) in rows {let files:Vec<String>=sqlx::query_scalar("SELECT filename FROM stored_files WHERE scope='message' AND parent_id=? ORDER BY created_at").bind(id).fetch_all(&pool).await?;results.push(serde_json::json!({"id":id,"user_id":user_id,"content":content,"created_at":created_at,"display_name":display_name,"avatar_url":avatar_url,"message_type":message_type,"edited_at":edited_at,"files":files}));}Ok(Json(results))
}
