use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::admin::models::JwtClaims;
use crate::auth::service::AuthService;
use crate::errors::AuthError;

/// JWT 认证中间件
pub async fn jwt_auth_middleware(
    State(auth_service): State<AuthService>,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // 提取 Authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or(AuthError::InvalidToken)?;

    match auth_service.validate_token(auth_header).await {
        Ok(claims) => {
            // 将用户信息注入请求扩展中
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(service_err) => match service_err {
            crate::errors::ServiceError::Auth(auth_err) => Err(auth_err),
            _ => Err(AuthError::InvalidToken),
        },
    }
}

/// 超级管理员权限中间件
pub async fn super_admin_middleware(req: Request, next: Next) -> Result<Response, AuthError> {
    let claims = req
        .extensions()
        .get::<JwtClaims>()
        .ok_or(AuthError::InvalidToken)?;

    if !claims.is_super_admin {
        return Err(AuthError::InsufficientPermissions);
    }

    Ok(next.run(req).await)
}

// 提取认证头的辅助函数（用于可选认证场景）
// pub fn extract_auth_header(req: &Request) -> Option<String> {
//     req.headers()
//         .get(AUTHORIZATION)?
//         .to_str()
//         .ok()?
//         .strip_prefix("Bearer ")
//         .map(|token| token.to_string())
// }
