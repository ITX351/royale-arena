mod admin;
mod auth;
mod config;
mod database;
mod director;
mod errors;
mod game;
mod routes;
mod rule_template;
mod websocket;

use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use admin::AdminService;
use auth::{AuthService, JwtManager};
use config::AppConfig;
use database::create_pool;
use director::DirectorService;
use game::{GameService, global_game_state_manager::GlobalGameStateManager};
use routes::create_routes;
use rule_template::RuleTemplateService;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "royale_arena_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = AppConfig::from_env()
        .expect("Failed to load configuration");

    // 创建数据库连接池
    let pool = create_pool(&config)
        .await
        .expect("Failed to create database pool");

    // 创建 JWT 管理器
    let jwt_manager = JwtManager::new(&config.jwt_secret, config.jwt_expiration_hours);

    // 创建服务实例
    let auth_service = AuthService::new(pool.clone(), jwt_manager);
    let admin_service = AdminService::new(pool.clone(), config.bcrypt_cost);
    let director_service = DirectorService::new(pool.clone());
    let game_service = GameService::new(pool.clone());
    let game_state_manager = GlobalGameStateManager::new(pool.clone());
    let rule_template_service = RuleTemplateService::new(pool.clone());
    // websocket_service 在路由中创建，不需要在这里创建

    // 构建路由
    let app = create_routes(
        auth_service, 
        admin_service, 
        director_service,
        game_service, 
        game_state_manager,
        rule_template_service,
        &config.api_prefix
    )
        .layer(TraceLayer::new_for_http());

    // 定义服务器地址
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!("server running on {} with API prefix: {}", addr, config.api_prefix);

    // 运行服务器
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}