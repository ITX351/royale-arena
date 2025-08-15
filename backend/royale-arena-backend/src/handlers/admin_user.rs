use crate::services::admin_service::{create_admin_user, get_admin_user};
use crate::services::db_helper::get_db_connection_from_pool;
use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateAdminUserRequest {
    pub username: String,
    pub password: String,
    pub is_super_admin: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct AdminUserResponse {
    pub id: String,
    pub username: String,
    pub is_super_admin: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CreateAdminUserResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<AdminUserResponse>,
}

/// 创建管理员用户
pub async fn create_admin_user_handler(
    create_request: web::Json<CreateAdminUserRequest>,
) -> Result<HttpResponse> {
    // Get database connection
    let mut conn = match get_db_connection_from_pool() {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Database connection error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(CreateAdminUserResponse {
                success: false,
                message: "Database connection error".to_string(),
                user: None,
            }));
        }
    };

    // Create admin user
    let is_super_admin = create_request.is_super_admin.unwrap_or(false);
    match create_admin_user(
        &mut conn,
        &create_request.username,
        &create_request.password,
        is_super_admin,
    ) {
        Ok(()) => {
            // Get the created user to return its ID
            match get_admin_user(&mut conn, &create_request.username) {
                Ok(Some(user)) => {
                    let user_response = AdminUserResponse {
                        id: user.id,
                        username: user.username,
                        is_super_admin: user.is_super_admin,
                    };
                    Ok(HttpResponse::Ok().json(CreateAdminUserResponse {
                        success: true,
                        message: "Admin user created successfully".to_string(),
                        user: Some(user_response),
                    }))
                }
                Ok(None) => {
                    tracing::error!("Failed to retrieve created admin user");
                    Ok(HttpResponse::InternalServerError().json(CreateAdminUserResponse {
                        success: false,
                        message: "Failed to retrieve created admin user".to_string(),
                        user: None,
                    }))
                }
                Err(e) => {
                    tracing::error!("Database query error: {}", e);
                    Ok(HttpResponse::InternalServerError().json(CreateAdminUserResponse {
                        success: false,
                        message: "Database query error".to_string(),
                        user: None,
                    }))
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to create admin user: {}", e);
            Ok(HttpResponse::InternalServerError().json(CreateAdminUserResponse {
                success: false,
                message: "Failed to create admin user".to_string(),
                user: None,
            }))
        }
    }
}