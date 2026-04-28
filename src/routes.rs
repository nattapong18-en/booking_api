use axum::{routing::{get, patch, post}, Router};
use sqlx::SqlitePool;

use crate::handles::{create_booking, get_bookings, cancel_booking, get_user_booking};

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/book", post(create_booking))
        .route("/bookings", get(get_bookings))
        .route("/cancel/{id}", patch(cancel_booking))
        .route("/bookings/user/{user_id}", get(get_user_booking))
        .with_state(pool)
}