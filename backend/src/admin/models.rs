use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 管理员用户实体
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminUser {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub is_super_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 管理员用户响应结构（不包含密码）
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUserResponse {
    pub id: String,
    pub username: String,
    pub is_super_admin: bool,
}

impl From<AdminUser> for AdminUserResponse {
    fn from(user: AdminUser) -> Self {
        Self {
            id: user.id,
            username: user.username,
            is_super_admin: user.is_super_admin,
        }
    }
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: String,
    pub expires_in: u64,
}

/// 创建管理员请求
#[derive(Debug, Deserialize)]
pub struct CreateAdminRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub is_super_admin: bool,
}

/// 更新管理员请求
#[derive(Debug, Deserialize)]
pub struct UpdateAdminRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_super_admin: Option<bool>,
}

/// 管理员列表响应
#[derive(Debug, Serialize)]
pub struct AdminListResponse {
    pub users: Vec<AdminUserResponse>,
}

/// 创建管理员响应
#[derive(Debug, Serialize)]
pub struct CreateAdminResponse {
    pub success: bool,
    pub message: String,
    pub user: AdminUserResponse,
}

/// 更新管理员响应
#[derive(Debug, Serialize)]
pub struct UpdateAdminResponse {
    pub success: bool,
    pub message: String,
    pub user: AdminUserResponse,
}

/// 删除管理员响应
#[derive(Debug, Serialize)]
pub struct DeleteAdminResponse {
    pub success: bool,
    pub message: String,
}

/// JWT Claims 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,        // user_id
    pub username: String,   // 用户名
    pub is_super_admin: bool, // 是否为超级管理员
    pub exp: usize,        // 过期时间戳
    pub iat: usize,        // 签发时间戳
}