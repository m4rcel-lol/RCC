use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub async fn setup_db() -> Pool<Postgres> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    // Create tables if they don't exist
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS game_instances (
            id UUID PRIMARY KEY,
            status TEXT NOT NULL,
            player_count INT DEFAULT 0
        )"
    ).execute(&pool).await.unwrap();

    pool
}
