use mysql::Opts;
use mysql::prelude::*;
use std::env;

pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_min: usize, // 连接池最小连接数
    pub pool_max: usize, // 连接池最大连接数
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("ROYALE_MYSQL_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("ROYALE_MYSQL_PORT")
                .unwrap_or_else(|_| "3306".to_string())
                .parse()
                .unwrap_or(3306),
            username: env::var("ROYALE_MYSQL_USER").unwrap_or_else(|_| "root".to_string()),
            password: env::var("ROYALE_MYSQL_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            database: env::var("ROYALE_MYSQL_DATABASE")
                .unwrap_or_else(|_| "royale_arena".to_string()),
            pool_min: env::var("ROYALE_MYSQL_POOL_MIN")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            pool_max: env::var("ROYALE_MYSQL_POOL_MAX")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

pub fn create_db_pool() -> mysql::Result<mysql::Pool> {
    let config = DatabaseConfig::from_env();

    // 创建连接选项
    let opts = Opts::from_url(&config.connection_string())?;

    let pool = mysql::Pool::new(opts)?;

    // 测试连接池是否正常工作
    let mut conn = pool.get_conn()?;
    match conn.query::<u32, _>("SELECT 1") {
        Ok(_) => {
            tracing::info!(
                "Database connection pool created successfully with min: {}, max: {} connections",
                config.pool_min,
                config.pool_max
            );
        }
        Err(e) => {
            tracing::error!("Failed to test database connection pool: {}", e);
            return Err(e);
        }
    }

    Ok(pool)
}
