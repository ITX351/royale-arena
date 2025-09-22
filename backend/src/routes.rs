use axum::{
    extract::ws::WebSocketUpgrade,
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::admin::{admin_login, create_admin, delete_admin, list_admins, update_admin};
use crate::auth::{jwt_auth_middleware, super_admin_middleware, AuthService};
use crate::admin::service::AdminService;
use crate::director::{batch_add_players, batch_delete_players, get_players, update_game_status, manual_save, list_save_files, DirectorService};
use crate::game::{create_game, delete_game, get_game_with_rules, get_games, get_player_messages, update_game, authenticate_game, GameService, GameLogService};
use crate::game::global_game_state_manager::GlobalGameStateManager;
use crate::rule_template::{create_template, get_templates, update_template, RuleTemplateService};
use crate::websocket::service::WebSocketService;
use crate::websocket::global_connection_manager::GlobalConnectionManager;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub admin_service: AdminService,
    pub director_service: DirectorService,
    pub game_service: GameService,
    pub game_log_service: GameLogService,
    pub game_state_manager: GlobalGameStateManager,
    pub rule_template_service: RuleTemplateService,
    pub global_connection_manager: GlobalConnectionManager,
}

pub fn create_routes(
    auth_service: AuthService, 
    admin_service: AdminService, 
    director_service: DirectorService,
    game_service: GameService,
    game_state_manager: GlobalGameStateManager,
    rule_template_service: RuleTemplateService,
    api_prefix: &str
) -> Router {
    let game_log_service = GameLogService::new(director_service.pool.clone());
    let global_connection_manager = GlobalConnectionManager::new();
    
    let app_state = AppState {
        auth_service: auth_service.clone(),
        admin_service,
        director_service: director_service.clone(),
        game_service,
        game_log_service,
        game_state_manager,
        rule_template_service,
        global_connection_manager,
    };

    // 公开路由（不需要认证）
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/admin/login", post(admin_login))
        // 规则模版公开查询接口
        .route("/rule-templates", get(get_templates))
        // 公开游戏查询接口
        .route("/games", get(get_games))
        .route("/games/{game_id}", get(get_game_with_rules))
        // WebSocket连接端点
        .route("/ws/{game_id}", get(
            |ws: WebSocketUpgrade, state: axum::extract::State<AppState>, path: axum::extract::Path<String>, query: axum::extract::Query<crate::websocket::models::WebSocketAuthRequest>| 
             WebSocketService::handle_websocket_upgrade(ws, state, path, query)
        ))
        .with_state(app_state.clone());

    // 需要超级管理员权限的路由
    let admin_routes = Router::new()
        .route("/users", get(list_admins))
        .route("/users", post(create_admin))
        .route("/users/{user_id}", put(update_admin))
        .route("/users/{user_id}", delete(delete_admin))
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
        .route("/", get(get_games))
        .route("/", post(create_game))
        .route("/{game_id}", get(get_game_with_rules))
        .route("/{game_id}", put(update_game))
        .route("/{game_id}", delete(delete_game))
        .layer(
            middleware::from_fn_with_state(
                auth_service.clone(),
                jwt_auth_middleware,
            )
        )
        .with_state(app_state.clone());

    // 需要管理员权限的规则模版路由
    let rule_template_admin_routes = Router::new()
        .route("/", get(get_templates))
        .route("/", post(create_template))
        .route("/{id}", put(update_template))
        .layer(
            middleware::from_fn_with_state(
                auth_service,
                jwt_auth_middleware,
            )
        )
        .with_state(app_state.clone());

    // 导演接口路由（无需JWT认证，使用导演密码验证）
    let director_routes = Router::new()
        .route("/game/{game_id}/players", 
               post(batch_add_players)
               .get(get_players)
               .delete(batch_delete_players))
        // 导演更新游戏状态接口
        .route("/game/{game_id}/status", put(update_game_status))
        // 手动存盘接口
        .route("/game/{game_id}/save", post(manual_save))
        // 查询存档文件列表接口
        .route("/game/{game_id}/saves", get(list_save_files))
        .with_state(app_state.clone());

    // 玩家接口路由（无需JWT认证，使用玩家密码验证）
    let player_routes = Router::new()
        // 获取玩家消息记录接口
        .route("/game/{game_id}/player/{player_id}/messages", post(get_player_messages))
        .with_state(app_state.clone());
        
    // 游戏认证路由
    let auth_routes = Router::new()
        .route("/game/{game_id}/auth", get(authenticate_game))
        .with_state(app_state.clone());

    // 组装 API 路由
    let api_routes = Router::new()
        .nest("/admin", admin_routes)
        .nest("/admin/games", game_admin_routes)
        .nest("/admin/rule-templates", rule_template_admin_routes)
        .merge(public_routes)
        .merge(director_routes)
        .merge(player_routes)
        .merge(auth_routes);

    // 最终应用路由
    Router::new().nest(api_prefix, api_routes)
}

// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}