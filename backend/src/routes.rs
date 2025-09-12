use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::admin::{admin_login, create_admin, delete_admin, list_admins, update_admin};
use crate::auth::{jwt_auth_middleware, super_admin_middleware, AuthService};
use crate::admin::service::AdminService;
use crate::director::{batch_add_players, batch_delete_players, get_players, DirectorService};
use crate::game::{create_game, delete_game, get_game_with_rules, get_games, update_game, GameService};
use crate::rule_template::{create_template, get_templates, update_template, RuleTemplateService};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub admin_service: AdminService,
    pub director_service: DirectorService,
    pub game_service: GameService,
    pub rule_template_service: RuleTemplateService,
}

pub fn create_routes(
    auth_service: AuthService, 
    admin_service: AdminService, 
    director_service: DirectorService,
    game_service: GameService,
    rule_template_service: RuleTemplateService,
    api_prefix: &str
) -> Router {
    let app_state = AppState {
        auth_service: auth_service.clone(),
        admin_service,
        director_service,
        game_service,
        rule_template_service,
    };

    // 公开路由（不需要认证）
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route(&format!("{}/admin/login", api_prefix), post(admin_login))
        // 规则模版公开查询接口
        .route(&format!("{}/rule-templates", api_prefix), get(get_templates))
        // 公开游戏查询接口
        .route(&format!("{}/games", api_prefix), get(get_games))
        .route(&format!("{}/games/{{game_id}}", api_prefix), get(get_game_with_rules))
        .with_state(app_state.clone());

    // 需要超级管理员权限的路由
    let admin_routes = Router::new()
        .route(&format!("{}/admin/users", api_prefix), get(list_admins))
        .route(&format!("{}/admin/users", api_prefix), post(create_admin))
        .route(&format!("{}/admin/users/{{user_id}}", api_prefix), put(update_admin))
        .route(&format!("{}/admin/users/{{user_id}}", api_prefix), delete(delete_admin))
        .layer(middleware::from_fn(super_admin_middleware))
        .layer(
            middleware::from_fn_with_state(
                auth_service.clone(),
                jwt_auth_middleware,
            )
        )
        .with_state(app_state.clone());

    // 需要管理员权限的游戏管理路由
    let game_admin_routes = Router::new()
        .route(&format!("{}/admin/games", api_prefix), get(get_games))
        .route(&format!("{}/admin/games", api_prefix), post(create_game))
        .route(&format!("{}/admin/games/{{game_id}}", api_prefix), get(get_game_with_rules))
        .route(&format!("{}/admin/games/{{game_id}}", api_prefix), put(update_game))
        .route(&format!("{}/admin/games/{{game_id}}", api_prefix), delete(delete_game))
        .layer(
            middleware::from_fn_with_state(
                auth_service.clone(),
                jwt_auth_middleware,
            )
        )
        .with_state(app_state.clone());

    // 需要管理员权限的规则模版路由
    let rule_template_admin_routes = Router::new()
        .route(&format!("{}/admin/rule-templates", api_prefix), get(get_templates))
        .route(&format!("{}/admin/rule-templates", api_prefix), post(create_template))
        .route(&format!("{}/admin/rule-templates/{{id}}", api_prefix), put(update_template))
        .layer(
            middleware::from_fn_with_state(
                auth_service,
                jwt_auth_middleware,
            )
        )
        .with_state(app_state.clone());

    // 导演接口路由（无需JWT认证，使用导演密码验证）
    let director_routes = Router::new()
        .route(&format!("{}/game/{{game_id}}/players", api_prefix), 
               post(batch_add_players)
               .get(get_players)
               .delete(batch_delete_players))
        .with_state(app_state.clone());

    // 合并路由
    Router::new()
        .merge(public_routes)
        .merge(admin_routes)
        .merge(game_admin_routes)
        .merge(rule_template_admin_routes)
        .merge(director_routes)
}

// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}