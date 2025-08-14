use std::env;

pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
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
            database: env::var("ROYALE_MYSQL_DATABASE").unwrap_or_else(|_| "royale_arena".to_string()),
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
    let opts = mysql::Opts::from_url(&config.connection_string())?;
    mysql::Pool::new(opts)
}