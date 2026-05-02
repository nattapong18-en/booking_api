mod routes;
mod models;
mod handles;
mod auth;
use sqlx::{sqlite::SqlitePoolOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    tracing::info!("Booking API Architecture Initialized!");  
    tracing_subscriber::fmt::init();
    
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = SqlitePoolOptions::new()
        .connect(&db_url)
        .await?;
    let app = routes::create_router(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    tracing::info!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;
    
    Ok(())    
}