mod auth;
mod getrooms;
mod handles;
mod models;
mod routes;
mod validate;

use tower_http::cors::{CorsLayer};
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use deadpool_redis::{Config, Runtime};



use crate::models::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Booking API Architecture Initialized!");

    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1".to_string());
    let mut redis_config = Config::from_url(&redis_url);
    redis_config.pool = Some(deadpool_redis::PoolConfig {
        max_size: 5,
        ..Default::default()
    });
    let redis_pool = redis_config.create_pool(Some(Runtime::Tokio1)).expect("Failed to create Redis pool");
    
    let options = db_url
        .parse::<PgConnectOptions>()?
        .ssl_mode(sqlx::postgres::PgSslMode::Require);

    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database migrations completed successfully!");
    let share_state = AppState {
        db: pool,
        jwt_secret,
        redis: redis_pool,
    };

    let app = routes::create_router(share_state)
              .layer(CorsLayer::permissive());
        
    let port = std::env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server running on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
