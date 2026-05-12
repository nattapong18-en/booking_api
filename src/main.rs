mod auth;
mod handles;
mod models;
mod routes;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::models::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Booking API Architecture Initialized!");

    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");
    if db_url.starts_with("sqlite:///") {
        let path = db_url
            .strip_prefix("sqlite://")
            .expect("Invalid DATABASE_URL format");
        if let Some(parent_dir) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent_dir).expect("Failed to create database directory")
        }
    }
    
    let connection_options = SqliteConnectOptions::from_str(&db_url)
    .expect("Invalid Database URL")
    .create_if_missing(true);
    let pool = SqlitePoolOptions::new().connect_with(connection_options).await?;
    sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("Failed to run database migrations");

    tracing::info!("Database migrations completed successfully!");
    let share_state = AppState {
        db: pool,
        jwt_secret,
    };

    let app = routes::create_router(share_state);
    let port = std::env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server running on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
