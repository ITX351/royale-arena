use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub bcrypt_cost: u32,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        dotenv().ok();
        
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL must be set in .env file".to_string())?;
            
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET must be set in .env file".to_string())?;
            
        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .map_err(|_| "JWT_EXPIRATION_HOURS must be a valid number".to_string())?;
            
        let bcrypt_cost = env::var("BCRYPT_COST")
            .unwrap_or_else(|_| "12".to_string())
            .parse()
            .map_err(|_| "BCRYPT_COST must be a valid number".to_string())?;
        
        Ok(Self {
            database_url,
            jwt_secret,
            jwt_expiration_hours,
            bcrypt_cost,
        })
    }
}