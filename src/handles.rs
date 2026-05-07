use axum::{Json, extract::State, http::StatusCode, response::IntoResponse, response::Response};

use crate::models::{
    AppError, AppState, BookingRecord, BookingState, Claims, CreateBookingRequest,DB_ERR_OVERLAP
};
use axum::extract::Path;

pub async fn get_bookings(
    State(state): State<AppState>,
) -> Result<Json<Vec<BookingRecord>>, AppError> {
    tracing::info!("Fetching all bookings...");

    let rows = sqlx::query!(
        "SELECT booking_id, user_id, room_id, start_time, end_time, status FROM bookings"
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
    tracing::info!("Cancelling booking id: {}", booking_id);
    let result = sqlx::query!(
        "UPDATE bookings SET status = 'Cancelled' WHERE booking_id = ? And user_id = ? And status = 'Confirmed'",
        booking_id,
        claims.user_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() > 0 {
        let mut rooms = state.available_rooms.lock().unwrap();
        *rooms += 1;
        Ok((
            StatusCode::OK,
            format!("Booking {} cancelled successfully", booking_id),
        )
            .into_response())
    } else {
        Err(AppError::NotFound(format!(
            "Booking {} not found",
            booking_id
        )))
    }
}

pub async fn get_user_booking(
    claims: Claims,
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<Json<Vec<BookingRecord>>, AppError> {
    if claims.user_id != user_id {
        return Err(AppError::Unauthorized(
            "You can only view your own booking".to_string(),
        ));
    }
    tracing::info!("Fetching bookings for user id: {}", user_id);

    let rows = sqlx::query!(
        "SELECT booking_id, user_id, room_id, start_time, end_time, status FROM bookings WHERE user_id = ?",
        user_id
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

pub async fn create_booking(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateBookingRequest>,
) -> Result<Response, AppError> {
    tracing::info!("Received booking request for room {}", payload.room_id);
    if payload.start_time >= payload.end_time {
        return Err(AppError::BadRequest("Invalid duration".to_string()));
    }

    {
        let mut rooms = state.available_rooms.lock().unwrap();
        if *rooms <= 0 {
            return Err(AppError::BadRequest("Room is full".to_string()));
        }
        *rooms -= 1;
        tracing::info!("Cache deducted. Remaining: {}", *rooms);
    }

    let db_process: Result::<Response, AppError> = async {
        let mut tx = state.db.begin().await?;
        let over_lap_check = sqlx::query!(
            "SELECT COUNT(*) as count FROM bookings WHERE room_id = ? AND status != 'Cancelled' AND start_time < ? AND end_time > ?",
            payload.room_id,
            payload.end_time,
            payload.start_time,
        )
        .fetch_one(&mut *tx)
        .await?;

        if over_lap_check.count > 0 {
            return Err(AppError::Conflict("Booking time overlaps with an existing booking".to_string()));
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
        let booking_id = insert_result.last_insert_rowid() as i32 ;
        let response = BookingRecord {
            booking_id,
            user_id: claims.user_id,
            room_id: payload.room_id,
            start_time: payload.start_time,
            end_time: payload.end_time,
            state: BookingState::Confirmed,
        };
        Ok(Json(response).into_response())
    }.await;

    if let Err(ref e) = db_process {
        let mut rooms = state.available_rooms.lock().unwrap();
        *rooms += 1;
        tracing::warn!(
            "Database error: {:?}. Room returned to cache. Total: {}",
            e,
            *rooms
        );
    }

    db_process
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_booking_invalid_time() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let state = AppState {
            db: pool,
            available_rooms: std::sync::Arc::new(std::sync::Mutex::new(100)),
        };
        let payload = serde_json::json!({
            "user_id": 1,
            "room_id":101,
            "start_time": "2020-01-01T10:00:00Z",
            "end_time": "2020-01-01T10:00:00Z"
        });
        let req = axum::http::Request::builder()
            .uri("/bookings")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(payload.to_string()))
            .unwrap();
        let app = axum::Router::new()
            .route("/bookings", axum::routing::post(create_booking))
            .with_state(state);

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST)
    }
}
