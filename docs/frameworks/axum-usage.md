# Axum框架使用指南

## 版本信息
- 使用版本: Axum 0.8.4

## 核心概念

### 1. 基本服务器设置
在Axum 0.8.4中，服务器启动方式已更新，不再使用`axum::Server::bind()`，而是使用`axum::serve()`函数：

```rust
use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // 构建应用路由
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // 绑定监听地址
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    // 启动服务器
    axum::serve(listener, app).await.unwrap();
}
```

### 2. 路由定义
使用`Router`和`routing`模块来定义路由：

```rust
use axum::{
    Router,
    routing::{get, post},
};

// 定义路由
let app = Router::new()
    .route("/", get(root_handler))
    .route("/users", get(list_users).post(create_user))
    .route("/users/:id", get(show_user).delete(delete_user));

async fn root_handler() { /* ... */ }
async fn list_users() { /* ... */ }
async fn create_user() { /* ... */ }
async fn show_user() { /* ... */ }
async fn delete_user() { /* ... */ }
```

### 3. 中间件
使用`tower_http`提供的中间件：

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new_for_http());
```

## 常见错误和解决方案

### 1. Server::bind() 已弃用
**错误信息**: `could not find Server in axum`

**解决方案**: 使用新的`axum::serve()`函数替代旧的`axum::Server::bind()`方法。

### 2. 运行时依赖
确保在`Cargo.toml`中包含了必要的依赖：

```toml
[dependencies]
axum = "0.8.4"
tokio = { version = "1.0", features = ["full"] }
tower-http = { version = "0.6.6", features = ["trace"] }
```

## 最佳实践

1. 使用`tower_http::trace::TraceLayer`进行请求追踪
2. 正确处理异步函数和错误
3. 使用类型安全的路由参数提取
4. 合理组织路由和中间件

## 参考资源
- [Axum官方文档](https://docs.rs/axum/0.8.4)
- [Axum示例代码](https://github.com/tokio-rs/axum/tree/main/examples)