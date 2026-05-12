use axum::{Json, extract::State, http::StatusCode, response::IntoResponse, response::Response};
use jsonwebtoken::{EncodingKey, Header};

use crate::models::{
    AppError, AppState, AuthResponse, BookingRecord, BookingState, Claims, CreateBookingRequest,
    DB_ERR_OVERLAP, LoginRequest, RegisterRequest, RegisterResponse, UserRow,
};
use axum::extract::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_my_bookings(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<BookingRecord>>, AppError> {
    tracing::info!("Fetching bookings for user id: {}", claims.user_id);

    let rows = sqlx::query!(
        "SELECT booking_id, user_id, room_id, start_time, end_time, status FROM bookings WHERE user_id = ?",
        claims.user_id
    )
    .fetch_all(&state.db)
    .await?;

    let bookings_list: Vec<BookingRecord> = rows
        .into_iter()
        .map(|row| {
            let current_state = match row.status.as_str() {
                "Confirmed" => BookingState::Confirmed,
                "Cancelled" => BookingState::Cancelled,
                _ => BookingState::Pending,
            };

            BookingRecord {
                booking_id: row.booking_id as i32,
                user_id: row.user_id as i32,
                room_id: row.room_id as i32,
                start_time: row.start_time.and_utc(),
                end_time: row.end_time.and_utc(),
                state: current_state,
            }
        })
        .collect();
    Ok(Json(bookings_list))
}
pub async fn cancel_booking(
    claims: Claims,
    State(state): State<AppState>,
    Path(booking_id): Path<i32>,
) -> Result<Response, AppError> {
    tracing::info!(
        "User {} is cancelling booking id: {}",
        claims.user_id,
        booking_id
    );
    let result = sqlx::query!(
        "UPDATE bookings SET status = 'Cancelled' WHERE booking_id = ? AND user_id = ? AND status = 'Confirmed'",
        booking_id,
        claims.user_id
        )
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Booking not found, already cancelled, or unauthorized".to_string(),
        ));
    }

    Ok(StatusCode::OK.into_response())
}

pub async fn create_booking(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateBookingRequest>,
) -> Result<Response, AppError> {
    tracing::info!("Received booking request for room {}", payload.room_id);
    if payload.start_time >= payload.end_time {
        return Err(AppError::BadRequest("Invalid duration".to_string()));
    }
    let mut tx = state.db.begin().await?;
    sqlx::query!("BEGIN IMMEDIATE").execute(&mut *tx).await?;

    let over_lap_check = sqlx::query!(
            "SELECT COUNT(*) as count FROM bookings WHERE room_id = ? AND status != 'Cancelled' AND start_time < ? AND end_time > ?",
            payload.room_id,
            payload.end_time,
            payload.start_time,
        )
        .fetch_one(&mut *tx)
        .await?;

    let max_capacity = 100;
    if over_lap_check.count >= max_capacity {
        return Err(AppError::Conflict(
            "Booking time overlaps with an existing booking (Room is full)".to_string(),
        ));
    }

    let insert_result = sqlx::query!(
            r#"INSERT INTO bookings (user_id, room_id, start_time, end_time, status) VALUES (?, ?, ?, ?, ?)"#,
            claims.user_id, payload.room_id, payload.start_time, payload.end_time, "Confirmed"
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.message().contains(DB_ERR_OVERLAP) {
                    return AppError::Conflict("This room is not free".to_string());
                }
            }
            AppError::from(e)
        })?;
    tx.commit().await?;
    let booking_id = insert_result.last_insert_rowid() as i32;
    let response = BookingRecord {
        booking_id,
        user_id: claims.user_id,
        room_id: payload.room_id,
        start_time: payload.start_time,
        end_time: payload.end_time,
        state: BookingState::Confirmed,
    };

    Ok(Json(response).into_response())
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    tracing::info!("User ID: {:?} is Login...", payload.username);
    let users = sqlx::query_as!(
        UserRow,
        r#"SELECT id as "id!: i32", password_hash FROM users WHERE username = ?"#,
        payload.username
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    let is_valid = bcrypt::verify(&payload.password, &users.password_hash)
        .map_err(|_| AppError::InternalServerError("Invalid username or password".to_string()))?;

    if !is_valid {
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + (24 * 60 * 60);

    let claims = Claims {
        user_id: users.id,
        exp: expiration,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::InternalServerError("Failed to encode token".to_string()))?;
    Ok(Json(AuthResponse { token }))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    tracing::info!("User: {} is registering...", payload.username);
    let hashed_password = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError("Invalid".to_string()))?;

    sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES (?,?)",
        payload.username,
        hashed_password,
    )
    .execute(&state.db)
    .await
    .map_err(|_| AppError::BadRequest("Username already exists".to_string()))?;

    Ok(Json(RegisterResponse {
        message: "Register successful".to_string(),
    }))
}
