use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::FromRow};

pub const DB_ERR_OVERLAP: &str = "ERR_OVERLAP";

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub jwt_secret: String,
}

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    Unauthorized(String),
    InternalServerError(String),
}

impl From<sqlx::Error> for AppError {
    fn from(inner: sqlx::Error) -> Self {
        AppError::DatabaseError(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.to_string()),
            AppError::InternalServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
            }
        };
        (
            status,
            axum::Json(serde_json::json!({"error": error_message})),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BookingState {
    Pending,
    Confirmed,
    Cancelled,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub room_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BookingRecord {
    pub booking_id: i32,
    pub user_id: i32,
    pub room_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub state: BookingState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(FromRow)]
pub struct UserRow {
    pub id: i32,
    pub password_hash: String,
}
