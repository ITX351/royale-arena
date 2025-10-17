use crate::config::AppConfig;
use sqlx::{MySql, Pool, mysql::MySqlPool};

pub type DatabasePool = Pool<MySql>;

pub async fn create_pool(config: &AppConfig) -> Result<DatabasePool, sqlx::Error> {
    MySqlPool::connect(&config.database_url).await
}
