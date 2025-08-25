use sqlx::{mysql::MySqlPool, MySql, Pool};
use crate::config::AppConfig;

pub type DatabasePool = Pool<MySql>;

pub async fn create_pool(config: &AppConfig) -> Result<DatabasePool, sqlx::Error> {
    MySqlPool::connect(&config.database_url).await
}