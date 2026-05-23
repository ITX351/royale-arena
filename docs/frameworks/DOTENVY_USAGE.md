# 📦 dotenvy 使用说明

`dotenvy` 是 [dotenv](https://crates.io/crates/dotenv) 的一个活跃维护分支，用于在 **Rust** 项目中从 `.env` 文件加载环境变量。  
它会在运行时读取 `.env` 文件中的配置，并将其合并进系统环境变量中，非常适合开发和测试环境。

---

## 🚀 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
dotenvy = "0.15"
```

---

## 📜 基本用法

### 1. 自动加载 `.env` 文件
在 `main.rs` 的入口初始化：
```rust
use dotenvy::dotenv;
use std::env;

fn main() {
    // 尝试从当前目录加载 .env 文件
    dotenv().ok();

    // 从环境变量中读取
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    println!("Database URL: {}", db_url);
}
```

### 2. 从自定义路径加载
```rust
use dotenvy::from_filename;
use std::env;

fn main() {
    from_filename("config/dev.env").ok();
    let token = env::var("API_TOKEN").unwrap();
    println!("Token: {}", token);
}
```

---

## ⚠️ 注意事项
- **不会覆盖已存在的系统环境变量**：如果系统中已经存在同名变量，`.env` 中的值会被忽略。
- **适用场景**：建议仅在开发和测试环境中使用，生产环境请通过安全的环境变量注入方式。
- 配合 `dotenvy_codegen` 宏可以在编译时读取 `.env` 值。

---

## 🔗 相关文档
- crates.io: [dotenvy](https://crates.io/crates/dotenvy)
- docs.rs: [dotenvy API 文档](https://docs.rs/dotenvy)
