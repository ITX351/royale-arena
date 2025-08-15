use bcrypt::{DEFAULT_COST, hash};
use mysql::prelude::*;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::from_filename(".env.royale")?;

    // Create database connection pool
    let config = crate::services::db::DatabaseConfig::from_env();
    let opts = mysql::Opts::from_url(&config.connection_string())?;
    let pool = mysql::Pool::new(opts)?;

    // Get connection from pool
    let mut conn = pool.get_conn()?;

    // Create admin_users table if it doesn't exist
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS admin_users (
            id VARCHAR(36) PRIMARY KEY,
            username VARCHAR(50) UNIQUE NOT NULL,
            password VARCHAR(255) NOT NULL,
            is_super_admin BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )",
    )?;

    // Insert super admin user (username: 'a', password: '1')
    let super_admin_id = Uuid::new_v4().to_string();
    let super_admin_password_hash = hash("1", DEFAULT_COST)?;
    conn.exec_drop(
        "INSERT IGNORE INTO admin_users (id, username, password, is_super_admin) VALUES (?, ?, ?, ?)",
        (&super_admin_id, "a", &super_admin_password_hash, true)
    )?;

    // Insert regular admin user (username: 'b', password: '2')
    let admin_id = Uuid::new_v4().to_string();
    let admin_password_hash = hash("2", DEFAULT_COST)?;
    conn.exec_drop(
        "INSERT IGNORE INTO admin_users (id, username, password, is_super_admin) VALUES (?, ?, ?, ?)",
        (&admin_id, "b", &admin_password_hash, false)
    )?;

    println!("Admin users initialized successfully!");
    println!("Super Admin - Username: a, Password: 1");
    println!("Regular Admin - Username: b, Password: 2");

    Ok(())
}

mod services {
    pub mod db {
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
                    password: env::var("ROYALE_MYSQL_PASSWORD")
                        .unwrap_or_else(|_| "password".to_string()),
                    database: env::var("ROYALE_MYSQL_DATABASE")
                        .unwrap_or_else(|_| "royale_arena".to_string()),
                }
            }

            pub fn connection_string(&self) -> String {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                )
            }
        }
    }
}
