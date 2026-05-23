use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

/// 认证相关错误
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
}

/// 服务层错误
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Auth error: {0}")]
    Auth(#[from] AuthError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Cannot delete super admin")]
    CannotDeleteSuperAdmin,

    #[error("User not found")]
    UserNotFound,

    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
}

/// 应用层错误
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal server error")]
    InternalServerError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "用户名或密码错误"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "用户不存在"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "认证令牌已过期"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "认证令牌无效"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "权限不足"),
            AuthError::JwtError(_) => (StatusCode::UNAUTHORIZED, "认证令牌处理失败"),
            AuthError::BcryptError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "密码处理失败"),
        };

        let body = Json(json!({
            "success": false,
            "error": message
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ServiceError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "数据库操作失败"),
            ServiceError::Auth(auth_err) => return auth_err.into_response(),
            ServiceError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ServiceError::UserAlreadyExists => (StatusCode::CONFLICT, "用户已存在"),
            ServiceError::CannotDeleteSuperAdmin => (StatusCode::FORBIDDEN, "不能删除超级管理员"),
            ServiceError::UserNotFound => (StatusCode::NOT_FOUND, "用户不存在"),
            ServiceError::Bcrypt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "密码处理失败"),
        };

        let body = Json(json!({
            "success": false,
            "error": message
        }));

        (status, body).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Service(service_err) => service_err.into_response(),
            AppError::Config(_) => {
                let body = Json(json!({
                    "success": false,
                    "error": "系统配置错误"
                }));
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            AppError::InternalServerError => {
                let body = Json(json!({
                    "success": false,
                    "error": "内部服务器错误"
                }));
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
        }
    }
}
