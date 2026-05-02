use axum::{routing::{get, patch, post}, Router};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;


use crate::handles::{create_booking, get_bookings, cancel_booking, get_user_booking};

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/book", post(create_booking))
        .route("/bookings", get(get_bookings))
        .route("/cancel/{id}", patch(cancel_booking))
        .route("/bookings/user/{user_id}", get(get_user_booking))
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}



#[cfg(test)]
mod tests {
    
    use tower::ServiceExt;
    use sqlx::sqlite::SqlitePoolOptions;

    use super::*;

        #[tokio::test] 
        async fn test_get_booking() {
        
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let req = axum::http::Request::builder().uri("/bookings").method("GET").body(axum::body::Body::empty()).unwrap();
        let app = axum::Router::new()
                   .route("/bookings", axum::routing::get(get_bookings))
                   .with_state(pool);
        let reponse = app.oneshot(req).await.unwrap();
        assert_eq!(reponse.status(),axum::http::StatusCode::OK);        
   } 
}   






