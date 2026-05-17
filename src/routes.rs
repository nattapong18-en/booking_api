use axum::{
    Router,
    routing::{get, patch, post},
};

use tower_http::trace::TraceLayer;

use crate::handles::{cancel_booking, create_booking, get_my_bookings, login, register, get_room};
use crate::models::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/book", post(create_booking))
        .route("/bookings", get(get_my_bookings))
        .route("/rooms", get(get_room))
        .route("/cancel/{id}", patch(cancel_booking))
        .route("/login", post(login))
        .route("/register", post(register))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
