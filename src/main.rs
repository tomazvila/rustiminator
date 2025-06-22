use rustimenator::{create_app, create_database_pool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./rustimenator.db".to_string());
    let pool = create_database_pool(&database_url).await?;
    let app = create_app(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");
    println!("Database: {}", database_url);

    axum::serve(listener, app).await?;

    Ok(())
}
