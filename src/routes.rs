use axum::{
    Router,
    routing::{get, patch, post},
};

use tower_http::trace::TraceLayer;

use crate::handles::{cancel_booking, create_booking, get_bookings, get_user_booking};
use crate::models::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/book", post(create_booking))
        .route("/bookings", get(get_bookings))
        .route("/cancel/{id}", patch(cancel_booking))
        .route("/bookings/user/{user_id}", get(get_user_booking))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {

    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_get_booking() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let state = AppState {
            db: pool,
            available_rooms: std::sync::Arc::new(std::sync::Mutex::new(100)),
        };
        sqlx::migrate!().run(&state.db).await.unwrap();
        let req = axum::http::Request::builder()
            .uri("/bookings")
            .method("GET")
            .body(axum::body::Body::empty())
            .unwrap();
        let app = axum::Router::new()
            .route("/bookings", axum::routing::get(get_bookings))
            .with_state(state);
        let reponse = app.oneshot(req).await.unwrap();
        assert_eq!(reponse.status(), axum::http::StatusCode::OK);
    }
}
