use sqlx::{MySql, MySqlPool, Pool};
use anyhow::Result;

pub type DbPool = Pool<MySql>;

pub async fn create_pool(database_url: &str) -> Result<DbPool> {
    let pool = MySqlPool::connect(database_url).await?;
    Ok(pool)
}