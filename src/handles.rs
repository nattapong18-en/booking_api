#![allow(dead_code)]
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{EncodingKey, Header};
use validator::Validate;

use crate::{
    getrooms::rooms_available,
    models::{
        AppError, AppState, AuthResponse, BookingRecord, BookingState, Claims,
        CreateBookingRequest, DB_ERR_OVERLAP, GetRoom, LoginRequest, RegisterRequest,
        RegisterResponse, RoomAvailability,UserRow,
    },
};
use axum::extract::Path;
use chrono::NaiveDate;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_my_bookings(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<BookingRecord>>, AppError> {
    tracing::info!("Fetching bookings for user id: {}", claims.user_id);

    let rows = sqlx::query!(
        "SELECT booking_id, user_id, room_id, start_time, end_time, status FROM bookings WHERE user_id = $1",
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
                start_time: row.start_time,
                end_time: row.end_time,
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
        "UPDATE bookings SET status = 'Cancelled' WHERE booking_id = $1 AND user_id = $2 AND status = 'Confirmed'",
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
    

    let over_lap_check = sqlx::query!(
            "SELECT COUNT(*) as count FROM bookings WHERE room_id = $1 AND status != 'Cancelled' AND start_time < $2 AND end_time > $3",
            payload.room_id,
            payload.end_time,
            payload.start_time,
        )
        .fetch_one(&mut *tx)
        .await?;

    let max_capacity = 1;
    let count = over_lap_check.count.unwrap_or(5);
    if count >= max_capacity {
        return Err(AppError::Conflict("
        booking time overlap with an existing booking"
        .to_string())
      );
    }

    let insert_result = sqlx::query_scalar!(
            r#"INSERT INTO bookings (user_id, room_id, start_time, end_time, status) VALUES ($1, $2, $3, $4, $5) RETURNING booking_id"#,
            claims.user_id, payload.room_id, payload.start_time, payload.end_time, "Confirmed"
        )
        .fetch_one(&mut *tx)
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
    let booking_id = insert_result;
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
        r#"SELECT id as "id!: i32", password_hash FROM users WHERE username = $1"#,
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
    payload.validate().map_err(|e| {
        let fields = e
            .field_errors()
            .iter()
            .map(|(field, error)| {
                let messages = error
                    .iter()
                    .filter_map(|e| e.message.as_deref())
                    .map(|s| s.to_string())
                    .collect();
                (field.to_string(), messages)
            })
            .collect();
        AppError::ValidationError(fields)
    })?;

    tracing::info!("User: {} is registering...", payload.username);

    let user_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
        payload.username
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(false);

    if user_exists  {
        return Err(AppError::Conflict("This users in used".to_string()));
    }

    let hashed_password = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
        .map_err(|_| AppError::InternalServerError("Invalid".to_string()))?;

    sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1,$2)",
        payload.username,
        hashed_password,
    )
    .execute(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                AppError::Conflict("Username already exists!".to_string())
            } else {
                AppError::InternalServerError("Database logic error".to_string())
            }
        }
        _ => AppError::InternalServerError("Datebase connection failed".to_string()),
    })?;

    Ok(Json(RegisterResponse {
        message: "Register successful".to_string(),
    }))
}

pub async fn get_room(
    State(state): State<AppState>,
    Query(payload): Query<GetRoom>,
) -> Result<Json<Vec<RoomAvailability>>, AppError> {
    tracing::info!("Get room: ...");
    let today = chrono::Local::now().date_naive();
    if payload.date_from < today {
        return Err(AppError::BadRequest("date_from must be today".to_string()));
    }
    
    if payload.date_to <= payload.date_from {
        return Err(AppError::BadRequest("date_to must be after date_from".to_string()));
    }

    let rooms = sqlx::query!("SELECT room_id FROM rooms")
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let mut result = Vec::new();
    for room in rooms {
        let bookings = sqlx::query!(
            "SELECT start_time, end_time FROM bookings WHERE room_id = $1",
            room.room_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let data_pair: Vec<(NaiveDate, NaiveDate)> = bookings
            .iter()
            .map(|b| {
                let check_in = b.start_time.date_naive();
                let check_out = b.end_time.date_naive();
                (check_in, check_out)
            })
            .collect();

        let status = if rooms_available(&data_pair, payload.date_from, payload.date_to) {
            "available"
        } else {
            "occupied"
        };

        result.push(RoomAvailability {
            room_id: room.room_id,
            status: status.to_string(),
        });
    }

    Ok(Json(result))
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::post,
    };
    use serde_json::json;
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    async fn setup_test_state() -> AppState {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL UNIQUE,
        password_hash TEXT NOT NULL
    );",
        )
        .execute(&pool)
        .await
        .unwrap();

        AppState {
            db: pool,
            jwt_secret: "my_jwt_secret".to_string(),
        }
    }
    #[tokio::test]
    async fn test_register_success() {
        let state = setup_test_state().await;

        let app = Router::new()
            .route("/register", post(register))
            .with_state(state);
        let payload = json!({
            "username": "Nattapong",
            "password": "12345679Bn"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_register_falid() {
        let state = setup_test_state().await;
        let app = Router::new()
            .route("/register", post(register))
            .with_state(state);
        let payload = json!({
            "username": "dd",
            "password": "1234"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
