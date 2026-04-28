use axum::{
    extract::{State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::SqlitePool;
use axum::extract::Path;    
use crate::models::{BookingRecord, BookingState, CreateBookingRequest};

// pub async fn create_booking(State(pool): State<SqlitePool>,Json(payload): Json<CreateBookingRequest>) -> Json<BookingRecord> {
//       println!("Received booking request for room {}: {:?}", payload.room_id, payload);
      
//       let insert_result = sqlx::query!(
//         r#"
//         INSERT INTO bookings (user_id, room_id, start_time, end_time, status) 
//         VALUES (?, ?, ?, ?, ?)"#,
//         payload.user_id,
//         payload.room_id,
//         payload.start_time,
//         payload.end_time,
//         "Pending"
//       )
//        .execute(&pool)
//        .await
//        .unwrap();
      
      
//       let booking_id = insert_result.last_insert_rowid() as i32;

//       let response = BookingRecord {
//         booking_id,
//         user_id: payload.user_id,
//         room_id: payload.room_id,
//         start_time: payload.start_time,
//         end_time:payload.end_time,
//         state: BookingState::Pending,
//       };
//       Json(response)
// }

pub async fn get_bookings(State(pool): State<SqlitePool>) -> Json<Vec<BookingRecord>> {
    println!("Fetching all bookings...");

    let rows = sqlx::query!(
        "SELECT booking_id, user_id, room_id, start_time, end_time, status FROM bookings"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
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
    Json(bookings_list)
}
pub async fn cancel_booking(State(pool): State<SqlitePool>, Path(booking_id):Path<i32>) -> impl IntoResponse {
        println!("Cancelling booking id: {}", booking_id);
        
        let result = sqlx::query!(
            "UPDATE bookings SET status = 'Cancelled' WHERE booking_id = ?",
            booking_id
        )
        .execute(&pool)
        .await
        .unwrap();
       
        if result.rows_affected() > 0 {
            return (StatusCode::OK, format!("Booking {} cancelled successfully", booking_id)).into_response();
        } else {
            return (StatusCode::NOT_FOUND, format!("Booking {} not found", booking_id)).into_response();
        }
        
        
}

pub async fn get_user_booking(State(pool): State<SqlitePool>, Path(user_id): Path<i32>) -> Json<Vec<BookingRecord>> {
    println!("Fetching bookings for user id: {}", user_id);

    let rows = sqlx::query!(
        "SELECT booking_id, user_id, room_id, start_time, end_time, status FROM bookings WHERE user_id = ?",
        user_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
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
    Json(bookings_list)
}

pub async fn create_booking(State(pool): State<SqlitePool>, Json(paload): Json<CreateBookingRequest>) -> impl IntoResponse {
        println!("Received booking request for room {}", paload.room_id);
        let mut tx = pool.begin().await.unwrap();
        
        let over_lap_check = sqlx::query!(
            "SELECT COUNT(*) as count FROM bookings WHERE room_id = ? AND status != 'Cancelled' AND start_time < ? AND end_time > ?",
            paload.room_id,
            paload.end_time,
            paload.start_time
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
      
        if over_lap_check.count > 0 {
            return (StatusCode::CONFLICT, "Booking time overlaps with an existing booking".to_string()).into_response();
        } else {
            let insert_result = sqlx::query!(
                r#"
                INSERT INTO bookings (user_id, room_id, start_time, end_time, status) 
                VALUES (?, ?, ?, ?, ?)"#,
                paload.user_id,
                paload.room_id,
                paload.start_time,
                paload.end_time,
                "Confirmed"
            )
            .execute(&mut *tx)
            .await
            .unwrap();
            
            tx.commit().await.unwrap();
            
            let booking_id = insert_result.last_insert_rowid() as i32;
            
            let response = BookingRecord {
                booking_id,
                user_id: paload.user_id,
                room_id: paload.room_id,
                start_time: paload.start_time,
                end_time: paload.end_time,
                state: BookingState::Confirmed,
            };
            return Json(response).into_response();
        }
        
}