mod routes;
mod models;
mod handles;
use sqlx::{sqlite::SqlitePoolOptions};

#[tokio::main]
async fn main() {
    println!("Booking API Architecture Initialized!");  

    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let pool = SqlitePoolOptions::new()
        .connect(&db_url)
        .await
        .unwrap(); 
    let app = routes::create_router(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();       
}

