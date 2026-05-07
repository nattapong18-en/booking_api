mod auth;
mod handles;
mod models;
mod routes;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::{Arc, Mutex};

use crate::models::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Booking API Architecture Initialized!");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = SqlitePoolOptions::new().connect(&db_url).await?;
    let share_state = AppState {
        db: pool,
        available_rooms: Arc::new(Mutex::new(100)),
    };

    let app = routes::create_router(share_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
