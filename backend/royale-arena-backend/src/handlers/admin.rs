use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;
use crate::models::admin::{LoginRequest, LoginResponse};

pub async fn admin_login(
    login_request: web::Json<LoginRequest>,
    data: web::Data<std::sync::Arc<tokio::sync::Mutex<crate::AppState>>>,
) -> Result<HttpResponse> {
    let state = data.lock().await;
    
    // Check if admin user exists and password matches
    // In a real application, passwords should be hashed
    if let Some(admin_user) = state.admin_users.get(&login_request.username) {
        if admin_user.password == login_request.password {
            // Generate a simple token (in production, use JWT or similar)
            let token = Uuid::new_v4().to_string();
            let expires_in = 3600; // 1 hour in seconds
            
            return Ok(HttpResponse::Ok().json(LoginResponse {
                success: true,
                token: Some(token),
                expires_in: Some(expires_in),
            }));
        }
    }
    
    // Invalid credentials
    Ok(HttpResponse::Unauthorized().json(LoginResponse {
        success: false,
        token: None,
        expires_in: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, http::StatusCode};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use crate::{AppState, models::admin::AdminUser};

    #[actix_web::test]
    async fn test_admin_login_success() {
        // Create test app state with an admin user
        let mut admin_users = HashMap::new();
        admin_users.insert(
            "admin".to_string(),
            AdminUser {
                username: "admin".to_string(),
                password: "password123".to_string(),
            },
        );
        
        let app_state = Arc::new(Mutex::new(AppState {
            games: HashMap::new(),
            admin_users,
        }));

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .route("/admin/login", web::post().to(admin_login))
        ).await;

        // Prepare login request
        let login_req = LoginRequest {
            username: "admin".to_string(),
            password: "password123".to_string(),
        };

        // Make request
        let req = test::TestRequest::post()
            .uri("/admin/login")
            .set_json(&login_req)
            .to_request();
            
        let resp = test::call_service(&app, req).await;

        // Check response
        assert_eq!(resp.status(), StatusCode::OK);
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json["success"].as_bool().unwrap());
        assert!(json["token"].is_string());
        assert_eq!(json["expires_in"].as_i64().unwrap(), 3600);
    }

    #[actix_web::test]
    async fn test_admin_login_invalid_username() {
        // Create test app state with an admin user
        let mut admin_users = HashMap::new();
        admin_users.insert(
            "admin".to_string(),
            AdminUser {
                username: "admin".to_string(),
                password: "password123".to_string(),
            },
        );
        
        let app_state = Arc::new(Mutex::new(AppState {
            games: HashMap::new(),
            admin_users,
        }));

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .route("/admin/login", web::post().to(admin_login))
        ).await;

        // Prepare login request with invalid username
        let login_req = LoginRequest {
            username: "invalid".to_string(),
            password: "password123".to_string(),
        };

        // Make request
        let req = test::TestRequest::post()
            .uri("/admin/login")
            .set_json(&login_req)
            .to_request();
            
        let resp = test::call_service(&app, req).await;

        // Check response
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(!json["success"].as_bool().unwrap());
        assert!(json["token"].is_null());
        assert!(json["expires_in"].is_null());
    }

    #[actix_web::test]
    async fn test_admin_login_invalid_password() {
        // Create test app state with an admin user
        let mut admin_users = HashMap::new();
        admin_users.insert(
            "admin".to_string(),
            AdminUser {
                username: "admin".to_string(),
                password: "password123".to_string(),
            },
        );
        
        let app_state = Arc::new(Mutex::new(AppState {
            games: HashMap::new(),
            admin_users,
        }));

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .route("/admin/login", web::post().to(admin_login))
        ).await;

        // Prepare login request with invalid password
        let login_req = LoginRequest {
            username: "admin".to_string(),
            password: "wrongpassword".to_string(),
        };

        // Make request
        let req = test::TestRequest::post()
            .uri("/admin/login")
            .set_json(&login_req)
            .to_request();
            
        let resp = test::call_service(&app, req).await;

        // Check response
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        
        let body = test::read_body(resp).await;
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(!json["success"].as_bool().unwrap());
        assert!(json["token"].is_null());
        assert!(json["expires_in"].is_null());
    }
}