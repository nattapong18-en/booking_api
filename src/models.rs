use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BookingState {
    Pending,
    Confirmed,
    Cancelled,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub user_id: i32,
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
