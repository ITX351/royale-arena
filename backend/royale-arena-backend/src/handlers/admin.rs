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
    let mut conn = match get_db_connection_from_pool() {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Database connection error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(LoginResponse {
                success: false,
                token: None,
                expires_in: None,
            }));
        }
    };

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
                tracing::warn!("Password verification failed: {}", e);
                return Ok(HttpResponse::Unauthorized().json(LoginResponse {
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web;
    use crate::test_common::test_init::init_test_env;

    #[actix_web::test]
    async fn test_admin_login_success() {
        // Initialize test environment
        init_test_env();
        
        // Create request data
        let request_data = LoginRequest {
            username: "a".to_string(),  // 使用测试数据中的管理员用户名
            password: "1".to_string(),  // 使用测试数据中的管理员密码
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = admin_login(json).await;
        
        // We expect a successful response since the test data should exist
        assert!(result.is_ok());
        
        // Check that we get a proper HTTP response
        let resp = result.unwrap();
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_admin_login_invalid_password() {
        // Initialize test environment
        init_test_env();
        
        // Create request data with invalid password
        let request_data = LoginRequest {
            username: "a".to_string(),  // 使用测试数据中的管理员用户名
            password: "invalid_password".to_string(),
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = admin_login(json).await;
        
        // We expect a successful response since the handler should handle the error internally
        assert!(result.is_ok());
        
        // Check that we get an unauthorized response
        let resp = result.unwrap();
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_admin_login_user_not_found() {
        // Initialize test environment
        init_test_env();
        
        // Create request data with non-existent user
        let request_data = LoginRequest {
            username: "non_existent_user".to_string(),
            password: "some_password".to_string(),
        };

        // Make request directly to the handler function
        let json = web::Json(request_data);
        let result = admin_login(json).await;
        
        // We expect a successful response since the handler should handle the error internally
        assert!(result.is_ok());
        
        // Check that we get an unauthorized response
        let resp = result.unwrap();
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }
}