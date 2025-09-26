use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::env;

pub async fn create_pool() -> Result<Pool<MySql>, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub struct Config {
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        Config {
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a number"),
        }
    }
}