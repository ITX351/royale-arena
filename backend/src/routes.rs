use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::admin::{admin_login, create_admin, delete_admin, list_admins, update_admin};
use crate::auth::{jwt_auth_middleware, super_admin_middleware, AuthService};
use crate::admin::service::AdminService;
use crate::rule_template::{create_template, get_templates, update_template, RuleTemplateService};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub admin_service: AdminService,
    pub rule_template_service: RuleTemplateService,
}

pub fn create_routes(auth_service: AuthService, admin_service: AdminService, rule_template_service: RuleTemplateService) -> Router {
    let app_state = AppState {
        auth_service: auth_service.clone(),
        admin_service,
        rule_template_service,
    };

    // 公开路由（不需要认证）
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/admin/login", post(admin_login))
        // 规则模版公开查询接口
        .route("/api/rule-templates", get(get_templates))
        .with_state(app_state.clone());

    // 需要超级管理员权限的路由
    let admin_routes = Router::new()
        .route("/api/admin/users", get(list_admins))
        .route("/api/admin/users", post(create_admin))
        .route("/api/admin/users/{user_id}", put(update_admin))
        .route("/api/admin/users/{user_id}", delete(delete_admin))
        .layer(middleware::from_fn(super_admin_middleware))
        .layer(
            middleware::from_fn_with_state(
                auth_service.clone(),
                jwt_auth_middleware,
            )
        )
        .with_state(app_state.clone());

    // 需要管理员权限的规则模版路由
    let rule_template_admin_routes = Router::new()
        .route("/api/admin/rule-templates", post(create_template))
        .route("/api/admin/rule-templates/{id}", put(update_template))
        .layer(
            middleware::from_fn_with_state(
                auth_service,
                jwt_auth_middleware,
            )
        )
        .with_state(app_state);

    // 合并路由
    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .merge(rule_template_admin_routes)
}

// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}