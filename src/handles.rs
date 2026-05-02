use axum::{
    extract::{State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;
use axum::extract::Path;    
use crate::models::{BookingRecord, BookingState, CreateBookingRequest, AppError, Claims};

pub async fn get_bookings(State(pool): State<SqlitePool>) -> Result<Json<Vec<BookingRecord>>, AppError> {
    tracing::info!("Fetching all bookings...");

    let rows = sqlx::query!(
        "SELECT booking_id, user_id, room_id, start_time, end_time, status FROM bookings"
    )
    .fetch_all(&pool)
    .await?;
    
    let mut bookings_list = Vec::new();

    for row in rows {
        let current_state = match row.status.as_str() {
            "Confirmed" => BookingState::Confirmed,
            "Cancelled" => BookingState::Cancelled,
            _ => BookingState::Pending,
        };

        bookings_list.push(BookingRecord {
            booking_id: row.booking_id as i32,
            user_id: row.user_id as i32,
            room_id: row.room_id as i32,
            start_time: row.start_time.and_utc(),
            end_time: row.end_time.and_utc()    ,
            state: current_state,
        });
    }
    Ok(Json(bookings_list))
}
pub async fn cancel_booking(claims:Claims ,State(pool): State<SqlitePool>, Path(booking_id):Path<i32>) -> Result<impl IntoResponse, AppError> {
        
        tracing::info!("Cancelling booking id: {}", booking_id);
        let result = sqlx::query!(
            "UPDATE bookings SET status = 'Cancelled' WHERE booking_id And user_id = ?",
            claims.user_id,
        )
        .execute(&pool)
        .await?;
        
       
        if result.rows_affected() > 0 {
             Ok((StatusCode::OK, format!("Booking {} cancelled successfully", booking_id)).into_response())
        } else {
             Err(AppError::NotFound(format!("Booking {} not found", booking_id)))
        }
        
        
}

pub async fn get_user_booking(State(pool): State<SqlitePool>, Path(user_id): Path<i32>) -> Result<Json<Vec<BookingRecord>>, AppError> {
    tracing::info!("Fetching bookings for user id: {}", user_id);

    let rows = sqlx::query!(
        "SELECT booking_id, user_id, room_id, start_time, end_time, status FROM bookings WHERE user_id = ?",
        user_id
    )
    .fetch_all(&pool)
    .await?;


    let mut bookings_list = Vec::new();

    for row in rows {
        let current_state = match row.status.as_str() {
            "Confirmed" => BookingState::Confirmed,
            "Cancelled" => BookingState::Cancelled,
            _ => BookingState::Pending,
        };
        bookings_list.push(BookingRecord {
            booking_id: row.booking_id as i32,
            user_id: row.user_id as i32,
            room_id: row.room_id as i32,
            start_time: row.start_time.and_utc(),
            end_time: row.end_time.and_utc(),
            state: current_state,
        });
    }
    Ok(Json(bookings_list))
}

pub async fn create_booking(State(pool): State<SqlitePool>, Json(payload): Json<CreateBookingRequest>) -> Result<impl IntoResponse, AppError> {
        tracing::info!("Received booking request for room {}", payload.room_id);
        if payload.start_time >= payload.end_time {
            return Err(AppError::BadRequest("Request is overlap".to_string()));
        } 
        
        let current_time = chrono::Utc::now();
        if payload.start_time < current_time {
            return Err(AppError::BadRequest("Time Error".to_string()));
        }
        
        let mut tx = pool.begin().await?;
        
        let over_lap_check = sqlx::query!(
            "SELECT COUNT(*) as count FROM bookings WHERE room_id = ? AND status != 'Cancelled' AND start_time < ? AND end_time > ?",
            payload.room_id,
            payload.end_time,
            payload.start_time
        )
        .fetch_one(&mut *tx)
        .await?;
        
      
        if over_lap_check.count > 0 {
             Err(AppError::Conflict("Booking time overlaps with an existing booking".to_string()))
        } else {
            let insert_result = sqlx::query!(
                r#"
                INSERT INTO bookings (user_id, room_id, start_time, end_time, status) 
                VALUES (?, ?, ?, ?, ?)"#,
                payload.user_id,
                payload.room_id,
                payload.start_time,
                payload.end_time,
                "Confirmed"
            )
            .execute(&mut *tx)
            .await?;
            
            
            tx.commit().await?;

            let booking_id = insert_result.last_insert_rowid() as i32;
            
            let response = BookingRecord {
                booking_id,
                user_id: payload.user_id,
                room_id: payload.room_id,
                start_time: payload.start_time,
                end_time: payload.end_time,
                state: BookingState::Confirmed,
            };
             Ok(Json(response).into_response())
        }
        
}


#[cfg(test)]
mod tests {
    use sqlx::{ sqlite::SqlitePoolOptions};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_booking_invalid_time(){
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let payload = serde_json::json!({
            "user_id": 1,
            "room_id":101,
            "start_time": "2020-01-01T10:00:00Z",
            "end_time": "2020-01-01T10:00:00Z"
        });
        let req = axum::http::Request::builder().uri("/bookings").method("POST").header("Content-Type", "application/json")
        .body(axum::body::Body::from(payload.to_string())).unwrap();
        let app = axum::Router::new()
        .route("/bookings", axum::routing::post(create_booking))
        .with_state(pool);
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST)
    }

    
}

