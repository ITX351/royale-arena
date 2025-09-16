use axum::{
    extract::{Path, State},
    Extension, Json,
};

use crate::admin::models::{
    AdminListResponse, CreateAdminRequest, CreateAdminResponse, DeleteAdminResponse,
    JwtClaims, LoginRequest, LoginResponse, UpdateAdminRequest, UpdateAdminResponse,
};
use crate::errors::ServiceError;
use crate::routes::AppState;

/// 管理员登录
pub async fn admin_login(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ServiceError> {
    let response = app_state.auth_service.login(request).await?;
    Ok(Json(response))
}

/// 获取管理员列表（仅超级管理员）
pub async fn list_admins(
    State(app_state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>, // 确保已通过认证
) -> Result<Json<AdminListResponse>, ServiceError> {
    let users = app_state.admin_service.list_admins().await?;
    Ok(Json(AdminListResponse { users }))
}

/// 创建管理员账户（仅超级管理员）
pub async fn create_admin(
    State(app_state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>, // 确保已通过认证
    Json(request): Json<CreateAdminRequest>,
) -> Result<Json<CreateAdminResponse>, ServiceError> {
    let user = app_state.admin_service.create_admin(request).await?;
    Ok(Json(CreateAdminResponse {
        success: true,
        message: "Admin user created successfully".to_string(),
        user,
    }))
}

/// 更新管理员账户（仅超级管理员）
pub async fn update_admin(
    State(app_state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>, // 确保已通过认证
    Path(user_id): Path<String>,
    Json(request): Json<UpdateAdminRequest>,
) -> Result<Json<UpdateAdminResponse>, ServiceError> {
    let user = app_state.admin_service.update_admin(&user_id, request).await?;
    Ok(Json(UpdateAdminResponse {
        success: true,
        message: "Admin user updated successfully".to_string(),
        user,
    }))
}

/// 删除管理员账户（仅超级管理员）
pub async fn delete_admin(
    State(app_state): State<AppState>,
    Extension(_claims): Extension<JwtClaims>, // 确保已通过认证
    Path(user_id): Path<String>,
) -> Result<Json<DeleteAdminResponse>, ServiceError> {
    app_state.admin_service.delete_admin(&user_id).await?;
    Ok(Json(DeleteAdminResponse {
        success: true,
        message: "Admin user deleted successfully".to_string(),
    }))
}