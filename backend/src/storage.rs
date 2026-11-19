use axum::{extract::{Multipart, Path, State, Extension, Query}, body::Body, http::{StatusCode, header}, response::Response};
use crate::models::error::AppError;
use tokio::io::{AsyncWriteExt,AsyncReadExt,AsyncSeekExt};
use uuid::Uuid;
use std::path::PathBuf;
use image::ImageOutputFormat;

pub const MAX_FILE: u64 = 800 * 1024 * 1024;
pub const MAX_REQUEST: usize = MAX_FILE as usize + 1024 * 1024;
pub const MAX_AVATAR: u64 = 2 * 1024 * 1024; // 2 MiB for avatars
pub const MAX_STICKER: u64 = 5 * 1024 * 1024; // 5 MiB for stickers
fn err(status: StatusCode, text: &str) -> AppError { AppError { status, message:text.into() } }
pub fn path(id: Uuid) -> PathBuf { PathBuf::from(std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".into())).join(id.to_string()) }
pub struct Upload { pub id:Uuid, pub filename:String, pub mime:String, pub size:i64 }
// Removes incomplete files, including when a client disconnects.
struct Temp(PathBuf, bool);
impl Drop for Temp { fn drop(&mut self) { if self.1 { let _=std::fs::remove_file(&self.0); } } }

// Compress an image file losslessly (PNG) or with quality setting (JPEG/WebP).
// Returns (new_size, new_mime) or None if compression not applicable.
fn compress_image(filepath: &std::path::Path, mime: &str) -> Result<Option<(u64, String)>, AppError> {
    let img = match image::open(filepath) {
        Ok(img) => img,
        Err(_) => return Ok(None),
    };
    let mut buf = Vec::new();
    let (fmt, ext) = match mime {
        "image/jpeg" => (ImageOutputFormat::Jpeg(85), "image/jpeg"),
        "image/webp" => (ImageOutputFormat::WebP(85), "image/webp"),
        "image/png" => (ImageOutputFormat::Png, "image/png"),
        "image/gif" => (ImageOutputFormat::Gif, "image/gif"),
        _ => return Ok(None),
    };
    if img.write_to(&mut buf, fmt).is_ok() && !buf.is_empty() {
        let _ = std::fs::write(filepath, &buf);
        Ok(Some((buf.len() as u64, ext.to_string())))
    } else {
        Ok(None)
    }
}
pub async fn receive(mut multipart: Multipart, image_only: bool) -> Result<Upload, AppError> {
 let mut field=multipart.next_field().await.map_err(|_|err(StatusCode::BAD_REQUEST,"Invalid upload."))?.ok_or_else(||err(StatusCode::BAD_REQUEST,"No file was provided."))?;
 let filename=field.file_name().unwrap_or("file").chars().filter(|c| !c.is_control() && *c!='/' && *c!='\\').take(200).collect::<String>();
 let claimed=field.content_type().unwrap_or("application/octet-stream").to_owned();
 if image_only && !["image/png","image/jpeg","image/gif","image/webp"].contains(&claimed.as_str()) {return Err(err(StatusCode::BAD_REQUEST,"Choose a PNG, JPEG, GIF or WebP image."));}
 let id=Uuid::new_v4(); let filepath=path(id); tokio::fs::create_dir_all(filepath.parent().unwrap()).await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not create upload directory."))?;
 let mut guard=Temp(filepath.clone(),true); let mut file=tokio::fs::File::create(&filepath).await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not store upload."))?;
 let mut size=0u64; let mut head=Vec::new();
 while let Some(chunk)=field.chunk().await.map_err(|_|err(StatusCode::BAD_REQUEST,"Upload was interrupted or too large."))? {
  size+=chunk.len() as u64; if size>MAX_FILE{return Err(err(StatusCode::PAYLOAD_TOO_LARGE,"File exceeds 800 MiB."));}
  if head.len()<16 {head.extend_from_slice(&chunk[..chunk.len().min(16-head.len())]);}
  file.write_all(&chunk).await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not write upload."))?;
 }
 if size==0{return Err(err(StatusCode::BAD_REQUEST,"File is empty."));}
 let actual=if head.starts_with(b"\x89PNG\r\n\x1a\n") {"image/png"} else if head.starts_with(b"\xff\xd8\xff") {"image/jpeg"} else if head.starts_with(b"GIF87a")||head.starts_with(b"GIF89a") {"image/gif"} else if head.len()>=12 && &head[..4]==b"RIFF" && &head[8..12]==b"WEBP" {"image/webp"} else if head.len()>=12 && &head[4..8]==b"ftyp" {if &head[8..12]==b"qt  " {"video/quicktime"} else {"video/mp4"}} else if head.starts_with(b"\x1a\x45\xdf\xa3") {"video/webm"} else if head.starts_with(b"OggS") && claimed.starts_with("video/") {"video/ogg"} else {"application/octet-stream"};
 if image_only && !actual.starts_with("image/") {return Err(err(StatusCode::BAD_REQUEST,"The file is not a supported image."));}
 file.flush().await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not finish upload."))?; guard.1=false;

 // Auto-compress images
 if actual.starts_with("image/") {
    if let Some((compressed_size, compressed_mime)) = compress_image(&filepath, &actual)? {
        size = compressed_size;
        let _ = std::fs::set_permissions(&filepath, std::fs::Permissions::from_mode(0o644));
        // Update mime if changed
        let final_mime = compressed_mime;
        Ok(Upload{id,filename:if filename.is_empty(){"file".into()}else{filename},mime:final_mime,size:size as i64})
    } else {
        Ok(Upload{id,filename:if filename.is_empty(){"file".into()}else{filename},mime:actual.into(),size:size as i64})
    }
 } else {
    Ok(Upload{id,filename:if filename.is_empty(){"file".into()}else{filename},mime:actual.into(),size:size as i64})
 }
}
pub async fn store(pool:&sqlx::SqlitePool, scope:&str, parent:Uuid, user:Uuid, upload:&Upload)->Result<(),AppError>{
 let result=sqlx::query("INSERT INTO stored_files (id,scope,parent_id,uploader_id,filename,mime,size) VALUES (?,?,?,?,?,?,?)").bind(upload.id).bind(scope).bind(parent).bind(user).bind(&upload.filename).bind(&upload.mime).bind(upload.size).execute(pool).await;
 if let Err(e)=result {let _=tokio::fs::remove_file(path(upload.id)).await;return Err(e.into());} Ok(())
}
pub async fn serve(id:Uuid,mime:&str)->Result<Response,AppError>{
 let file=tokio::fs::File::open(path(id)).await.map_err(|_|err(StatusCode::NOT_FOUND,"File not found."))?;
 let disposition=if mime.starts_with("image/")||mime.starts_with("video/"){"inline"}else{"attachment; filename=\"download\""};
 Response::builder().header(header::CONTENT_TYPE,mime).header(header::CONTENT_DISPOSITION,disposition).header("X-Content-Type-Options","nosniff").header("Referrer-Policy","no-referrer").header(header::CACHE_CONTROL,"private, no-store").body(Body::from_stream(tokio_util::io::ReaderStream::new(file))).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not read file."))
}
pub async fn avatar(State(pool):State<sqlx::SqlitePool>,Path(user):Path<Uuid>)->Result<Response,AppError>{
 let row:Option<(Uuid,String)>=sqlx::query_as("SELECT id,mime FROM stored_files WHERE scope='avatar' AND parent_id=? ORDER BY created_at DESC LIMIT 1").bind(user).fetch_optional(&pool).await?;
 let (id,mime)=row.ok_or_else(||err(StatusCode::NOT_FOUND,"Image not found."))?; serve(id,&mime).await
}

async fn permitted_file(pool:&sqlx::SqlitePool,id:Uuid,user:Uuid)->Result<String,AppError>{
 let row:Option<(String,Uuid,String)>=sqlx::query_as("SELECT scope,parent_id,mime FROM stored_files WHERE id=?").bind(id).fetch_optional(pool).await?;
 let (scope,parent,mime)=row.ok_or_else(||err(StatusCode::NOT_FOUND,"File not found."))?;
 if scope=="sticker" {return Ok(mime);}
 let channel:Option<Uuid>=match scope.as_str(){"issue"=>sqlx::query_scalar("SELECT channel_id FROM issues WHERE id=?").bind(parent).fetch_optional(pool).await?,"message"=>sqlx::query_scalar("SELECT channel_id FROM messages WHERE id=?").bind(parent).fetch_optional(pool).await?,_=>None};
 let channel=channel.ok_or_else(||err(StatusCode::NOT_FOUND,"File not found."))?;
 crate::access::channel_access(pool,channel,user).await?;Ok(mime)
}
pub async fn ticket(State(pool):State<sqlx::SqlitePool>,Extension(user):Extension<Uuid>,Path(id):Path<Uuid>)->Result<axum::Json<serde_json::Value>,AppError>{
 permitted_file(&pool,id,user).await?;
 let secret=std::env::var("JWT_SECRET").unwrap_or_else(|_|"default_secret_change_me".into());
 let claims=serde_json::json!({"sub":user,"file":id,"kind":"file_download","exp":(chrono::Utc::now()+chrono::Duration::minutes(5)).timestamp()});
 let token=jsonwebtoken::encode(&jsonwebtoken::Header::default(),&claims,&jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not create file link."))?;
 Ok(axum::Json(serde_json::json!({"url":format!("/api/file-download/{}",token)})))
}
#[derive(serde::Deserialize)] pub struct DownloadQuery { download: Option<u8> }
pub async fn ticket_download(State(pool):State<sqlx::SqlitePool>,Path(token):Path<String>,Query(query):Query<DownloadQuery>,headers:axum::http::HeaderMap)->Result<Response,AppError>{
 let secret=std::env::var("JWT_SECRET").unwrap_or_else(|_|"default_secret_change_me".into());
 let claims:jsonwebtoken::TokenData<serde_json::Value>=jsonwebtoken::decode(&token,&jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),&jsonwebtoken::Validation::default()).map_err(|_|err(StatusCode::UNAUTHORIZED,"File link expired."))?;
 if claims.claims["kind"]!="file_download"{return Err(err(StatusCode::UNAUTHORIZED,"Invalid file link."));}
 let user=claims.claims["sub"].as_str().and_then(|v|Uuid::parse_str(v).ok()).ok_or_else(||err(StatusCode::UNAUTHORIZED,"Invalid file link."))?;
 let id=claims.claims["file"].as_str().and_then(|v|Uuid::parse_str(v).ok()).ok_or_else(||err(StatusCode::UNAUTHORIZED,"Invalid file link."))?;
 let active:Option<bool>=sqlx::query_scalar("SELECT is_active FROM users WHERE id=?").bind(user).fetch_optional(&pool).await?;if active!=Some(true){return Err(err(StatusCode::UNAUTHORIZED,"Account is inactive."));}
 let mut mime=permitted_file(&pool,id,user).await?;
 if mime=="application/octet-stream" { // Existing uploads predate video detection.
  if let Ok(mut file)=tokio::fs::File::open(path(id)).await {let mut head=[0u8;16];let n=file.read(&mut head).await.unwrap_or(0);let bytes=&head[..n];
   if n>=12&&&bytes[4..8]==b"ftyp" {mime=if &bytes[8..12]==b"qt  " {"video/quicktime"}else{"video/mp4"}.into();}
   else if bytes.starts_with(b"\x1a\x45\xdf\xa3"){mime="video/webm".into();}
   else if bytes.starts_with(b"OggS"){let name:Option<String>=sqlx::query_scalar("SELECT filename FROM stored_files WHERE id=?").bind(id).fetch_optional(&pool).await?;if name.is_some_and(|n|n.to_lowercase().ends_with(".ogv")){mime="video/ogg".into();}}
  }
 }
 let mut response=if let Some(range)=headers.get(header::RANGE).and_then(|v|v.to_str().ok()) {
  let mut file=tokio::fs::File::open(path(id)).await.map_err(|_|err(StatusCode::NOT_FOUND,"File not found."))?;
  let size=file.metadata().await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not read file."))?.len();
  let spec=range.strip_prefix("bytes=").and_then(|r|r.split_once('-')).ok_or_else(||err(StatusCode::RANGE_NOT_SATISFIABLE,"Invalid range."))?;
  let (start,end)=if spec.0.is_empty(){let n=spec.1.parse::<u64>().map_err(|_|err(StatusCode::RANGE_NOT_SATISFIABLE,"Invalid range."))?;(size.saturating_sub(n),size.saturating_sub(1))}else{(spec.0.parse::<u64>().map_err(|_|err(StatusCode::RANGE_NOT_SATISFIABLE,"Invalid range."))?,if spec.1.is_empty(){size.saturating_sub(1)}else{spec.1.parse::<u64>().map_err(|_|err(StatusCode::RANGE_NOT_SATISFIABLE,"Invalid range."))?.min(size.saturating_sub(1))})};
  if size==0||start>=size||end<start {return Err(err(StatusCode::RANGE_NOT_SATISFIABLE,"Invalid range."));}file.seek(std::io::SeekFrom::Start(start)).await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not seek file."))?;
  Response::builder().status(StatusCode::PARTIAL_CONTENT).header(header::CONTENT_TYPE,&mime).header(header::CONTENT_LENGTH,(end-start+1).to_string()).header(header::CONTENT_RANGE,format!("bytes {}-{}/{}",start,end,size)).header(header::ACCEPT_RANGES,"bytes").header("X-Content-Type-Options","nosniff").header(header::CACHE_CONTROL,"private, no-store").body(Body::from_stream(tokio_util::io::ReaderStream::new(file.take(end-start+1)))).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"Could not read file."))?
 }else{let mut r=serve(id,&mime).await?;r.headers_mut().insert(header::ACCEPT_RANGES,axum::http::HeaderValue::from_static("bytes"));r};
 if query.download==Some(1) {
  let name:String=sqlx::query_scalar("SELECT filename FROM stored_files WHERE id=?").bind(id).fetch_one(&pool).await?;
  let fallback:String=name.chars().filter(|c|c.is_ascii_alphanumeric()||" ._-".contains(*c)).take(120).collect();
  let fallback=if fallback.is_empty(){"download"}else{fallback.as_str()};
  let encoded:String=name.as_bytes().iter().map(|b|if b.is_ascii_alphanumeric()||b"-._~".contains(b){(*b as char).to_string()}else{format!("%{:02X}",b)}).collect();
  let disposition=format!("attachment; filename=\"{}\"; filename*=UTF-8''{}",fallback,encoded);
  if let Ok(value)=axum::http::HeaderValue::from_str(&disposition){response.headers_mut().insert(header::CONTENT_DISPOSITION,value);}
 }
 Ok(response)
}

pub async fn team_avatar(State(pool):State<sqlx::SqlitePool>,Path(team):Path<Uuid>)->Result<Response,AppError>{
 let row:Option<(Uuid,String)>=sqlx::query_as("SELECT id,mime FROM stored_files WHERE scope='team_avatar' AND parent_id=? ORDER BY rowid DESC LIMIT 1").bind(team).fetch_optional(&pool).await?;
 let (id,mime)=row.ok_or_else(||err(StatusCode::NOT_FOUND,"Image not found."))?;serve(id,&mime).await
}
