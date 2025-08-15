use crate::models::admin::{LoginRequest, LoginResponse};
use crate::services::db_helper::get_db_connection_from_pool;
use actix_web::{HttpResponse, Result, web};
use bcrypt::verify;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use mysql::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String, // subject (user ID)
    exp: usize,  // expiration time
    is_super_admin: bool,
}

pub async fn admin_login(login_request: web::Json<LoginRequest>) -> Result<HttpResponse> {
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
    if let Some((user_id, hashed_password, is_super_admin)) = admin_user {
        // Use bcrypt to verify password
        match verify(&login_request.password, &hashed_password) {
            Ok(true) => {
                // Generate JWT token
                let expiration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as usize
                    + 3600; // 1 hour expiration

                let claims = Claims {
                    sub: user_id,
                    exp: expiration,
                    is_super_admin,
                };

                // Load secret key from environment or use default for demo
                let secret = std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "royale_arena_secret".to_string());

                match encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(secret.as_ref()),
                ) {
                    Ok(token) => {
                        return Ok(HttpResponse::Ok().json(LoginResponse {
                            success: true,
                            token: Some(token),
                            expires_in: Some(3600),
                        }));
                    }
                    Err(e) => {
                        tracing::error!("JWT token generation error: {}", e);
                        return Ok(HttpResponse::InternalServerError().json(LoginResponse {
                            success: false,
                            token: None,
                            expires_in: None,
                        }));
                    }
                }
            }
            Ok(false) => {
                // Password doesn't match
                return Ok(HttpResponse::Unauthorized().json(LoginResponse {
                    success: false,
                    token: None,
                    expires_in: None,
                }));
            }
            Err(e) => {
                tracing::error!("Password verification error: {}", e);
                return Ok(HttpResponse::InternalServerError().json(LoginResponse {
                    success: false,
                    token: None,
                    expires_in: None,
                }));
            }
        }
    }

    // User not found
    Ok(HttpResponse::Unauthorized().json(LoginResponse {
        success: false,
        token: None,
        expires_in: None,
    }))
}

/// 验证JWT令牌
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "royale_arena_secret".to_string());
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;
    Ok(token_data.claims)
}
