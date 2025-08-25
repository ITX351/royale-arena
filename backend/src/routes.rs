use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::admin::{admin_login, create_admin, delete_admin, list_admins, update_admin};
use crate::auth::{jwt_auth_middleware, super_admin_middleware, AuthService};
use crate::admin::service::AdminService;

pub fn create_routes(auth_service: AuthService, admin_service: AdminService) -> Router {
    // 公开路由（不需要认证）
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/admin/login", post(admin_login))
        .with_state(auth_service.clone());

    // 需要超级管理员权限的路由
    let admin_routes = Router::new()
        .route("/api/admin/users", get(list_admins))
        .route("/api/admin/users", post(create_admin))
        .route("/api/admin/users/:user_id", put(update_admin))
        .route("/api/admin/users/:user_id", delete(delete_admin))
        .layer(middleware::from_fn(super_admin_middleware))
        .layer(
            middleware::from_fn_with_state(
                auth_service.clone(),
                jwt_auth_middleware,
            )
        )
        .with_state(admin_service);

    // 合并路由
    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
}

// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}