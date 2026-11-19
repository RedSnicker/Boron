use crate::models::user::{
    CreateUserRequest, LoginRequest, User, UserRole, AuthResponse, UserResponse,
};
use sqlx::SqlitePool;

pub struct AuthService {
    pool: SqlitePool,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_secret_change_me".into());
        
        Self { pool, jwt_secret }
    }

    pub async fn register(
        &self,
        payload: CreateUserRequest,
    ) -> Result<AuthResponse, ServiceError> {
        if payload.email.trim().is_empty() || payload.password.len() < 8 || payload.display_name.trim().is_empty() {
            return Err(ServiceError { status_code: axum::http::StatusCode::BAD_REQUEST, message: "Email, a username, and a password of at least 8 characters are required.".into() });
        }
        if let Some(ref image) = payload.avatar_url {
            if image.len() > 700_000 || !["data:image/png;base64,", "data:image/jpeg;base64,", "data:image/webp;base64,", "data:image/gif;base64,"].iter().any(|prefix| image.starts_with(prefix)) {
                return Err(ServiceError { status_code: axum::http::StatusCode::BAD_REQUEST, message: "Profile picture must be a PNG, JPEG, GIF or WebP under 500 KB.".into() });
            }
        }
        let registration_mode = std::env::var("ACCOUNT_REGISTRATION_MODE")
            .unwrap_or_else(|_| "open".into());
        


        let user_count: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ServiceError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Failed to check user count: {}", e),
            })?;

        let is_active = registration_mode != "closed" || matches!(user_count, Some((0,)));
        let role = match user_count {
            Some((count,)) if count == 0 => UserRole::Admin,
            _ => UserRole::Member,
        };

        let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
            .map_err(|e| ServiceError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Failed to hash password: {}", e),
            })?;

        let user_id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now().naive_utc();

        sqlx::query("INSERT INTO users (id, email, password_hash, display_name, avatar_url, role, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&user_id)
            .bind(&payload.email)
            .bind(&password_hash)
            .bind(&payload.display_name)
            .bind(&payload.avatar_url)
            .bind(role.clone())
            .bind(is_active)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| ServiceError {
                status_code: if e.as_database_error().is_some_and(|d| d.is_unique_violation()) { axum::http::StatusCode::CONFLICT } else { axum::http::StatusCode::INTERNAL_SERVER_ERROR },
                message: if e.as_database_error().is_some_and(|d| d.is_unique_violation()) { "An account with that email already exists.".into() } else { format!("Failed to create user: {}", e) },
            })?;

        let user = User {
            id: user_id,
            email: payload.email,
            password_hash,
            display_name: payload.display_name,
            avatar_url: payload.avatar_url,
            bio: String::new(),
            role,
            is_active,
            created_at: now,
            updated_at: now,
        };

        if !user.is_active { return Err(ServiceError { status_code: axum::http::StatusCode::FORBIDDEN, message: "Account created; administrator approval is required before signing in.".into() }); }
        let token = self.generate_token(&user)?;
        
        Ok(AuthResponse {
            token,
            user: UserResponse::from(user),
        })
    }

    pub async fn login(
        &self,
        payload: LoginRequest,
    ) -> Result<AuthResponse, ServiceError> {
        let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = ?")
            .bind(&payload.email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ServiceError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Database error: {}", e),
            })?;

        let user = user.ok_or_else(|| ServiceError {
            status_code: axum::http::StatusCode::UNAUTHORIZED,
            message: "Invalid email or password".into(),
        })?;

        if !user.is_active {
            return Err(ServiceError {
                status_code: axum::http::StatusCode::FORBIDDEN,
                message: "Account is not active. Please contact an administrator.".into(),
            });
        }

        let valid = bcrypt::verify(&payload.password, &user.password_hash)
            .unwrap_or(false);

        if !valid {
            return Err(ServiceError {
                status_code: axum::http::StatusCode::UNAUTHORIZED,
                message: "Invalid email or password".into(),
            });
        }

        let token = self.generate_token(&user)?;
        
        Ok(AuthResponse {
            token,
            user: UserResponse::from(user),
        })
    }

    pub async fn get_user(&self, user_id: uuid::Uuid) -> Result<User, ServiceError> {
        let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ServiceError {
                status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Database error: {}", e),
            })?;

        user.ok_or_else(|| ServiceError {
            status_code: axum::http::StatusCode::NOT_FOUND,
            message: "User not found".into(),
        })
    }

    fn generate_token(&self, user: &User) -> Result<String, ServiceError> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::seconds(
            std::env::var("JWT_EXPIRATION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2592000)
        );

        let claims = serde_json::json!({
            "sub": user.id,
            "email": user.email,
            "role": user.role,
            "exp": exp.timestamp(),
        });

        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| ServiceError {
            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Failed to generate token: {}", e),
        })
    }
}

#[derive(Debug)]
pub struct ServiceError {
    pub status_code: axum::http::StatusCode,
    pub message: String,
}

impl axum::response::IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, self.message).into_response()
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(e: sqlx::Error) -> Self {
        ServiceError {
            status_code: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Database error: {}", e),
        }
    }
}