use crate::models::admin::{LoginRequest, LoginResponse};
use crate::services::admin_service::{get_admin_user, verify_admin_password};
use crate::services::db_helper::get_db_connection_from_pool;
use actix_web::{HttpResponse, Result, web};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
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

    // Get admin user from database using the service function
    let admin_user = match get_admin_user(&mut conn, &login_request.username) {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Database query error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(LoginResponse {
                success: false,
                token: None,
                expires_in: None,
            }));
        }
    };

    // Check if user exists
    if let Some(user) = admin_user {
        // Verify password using the service function
        let password_valid = match verify_admin_password(&mut conn, &login_request.username, &login_request.password) {
            Ok(valid) => valid,
            Err(e) => {
                tracing::error!("Password verification error: {}", e);
                return Ok(HttpResponse::InternalServerError().json(LoginResponse {
                    success: false,
                    token: None,
                    expires_in: None,
                }));
            }
        };

        if password_valid {
            // Generate JWT token
            let expiration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as usize
                + 3600; // 1 hour expiration

            let claims = Claims {
                sub: user.id,
                exp: expiration,
                is_super_admin: user.is_super_admin,
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
        } else {
            // Password doesn't match
            return Ok(HttpResponse::Unauthorized().json(LoginResponse {
                success: false,
                token: None,
                expires_in: None,
            }));
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
