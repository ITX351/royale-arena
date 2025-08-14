use actix_web::{web, HttpResponse, Result};
use crate::models::admin::{LoginRequest, LoginResponse};
use crate::services::db_helper::get_db_connection_from_pool;
use mysql::prelude::*;

pub async fn admin_login(
    login_request: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    // Get database connection
    let mut conn = get_db_connection_from_pool()?;
    
    // Query admin user from database
    let admin_user: Option<(String, String, bool)> = conn
        .exec_first(
            "SELECT id, password, is_super_admin FROM admin_users WHERE username = ?",
            (&login_request.username,),
        )
        .map_err(|e| {
            tracing::error!("Database query error: {}", e);
            actix_web::error::ErrorInternalServerError("Database query error")
        })?;
    
    // Check if user exists and password matches
    if let Some((_user_id, hashed_password, _is_super_admin)) = admin_user {
        // In a real application, you should use a proper password hashing library like bcrypt
        // For now, we're doing a simple comparison, but this should be replaced
        if hashed_password == login_request.password {
            // Generate a simple token (in production, use JWT or similar)
            let token = uuid::Uuid::new_v4().to_string();
            let expires_in = 3600; // 1 hour in seconds
            
            return Ok(HttpResponse::Ok().json(LoginResponse {
                success: true,
                token: Some(token),
                expires_in: Some(expires_in),
            }));
        }
    }
    
    // User not found or invalid credentials
    Ok(HttpResponse::Unauthorized().json(LoginResponse {
        success: false,
        token: None,
        expires_in: None,
    }))
}