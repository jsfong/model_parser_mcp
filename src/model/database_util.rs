
use sqlx::postgres::PgPoolOptions;
use std::{env, time::Duration};

/* DB */
pub async fn connect_to_db() -> sqlx::Pool<sqlx::Postgres> {

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))//Max time to wait for a connection
        .idle_timeout(Duration::from_secs(60)) //Max idle time for connection in a pool
        .max_lifetime(Duration::from_secs(3600)) // Max lifetime for a connection
        .connect(&database_url)
        .await
        .expect("Failed to create DB pool.");

    println!("Connected to the database");
    pg_pool
}
