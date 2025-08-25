use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // 构建我们的应用路由
    let app = Router::new()
        .route("/", get(handler))
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http());

    // 定义服务器地址
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("server running on {}", addr);

    // 运行服务器 (修正Axum 0.8的语法)
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 基础的处理函数
async fn handler() -> &'static str {
    "Welcome to Royale Arena Backend!"
}

// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}