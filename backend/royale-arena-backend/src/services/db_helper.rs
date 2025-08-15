use crate::services::db::create_db_pool;
use actix_web::error::ErrorInternalServerError;
use mysql::PooledConn;

/// Creates a database connection pool and gets a connection with standardized error handling
pub fn get_db_connection_from_pool() -> Result<PooledConn, actix_web::Error> {
    // Create database connection pool
    let pool = create_db_pool().map_err(|e| {
        tracing::error!("Failed to create database pool: {}", e);
        ErrorInternalServerError("Database connection error")
    })?;

    // Get connection from pool
    let conn = pool.get_conn().map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        ErrorInternalServerError("Database connection error")
    })?;

    Ok(conn)
}
