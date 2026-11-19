use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, axum::Json(serde_json::json!({"message": self.message}))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Database error: {}", e),
        }
    }
}