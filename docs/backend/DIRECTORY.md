# Royale Arena 后端目录结构

## 根目录
backend/
├── Cargo.toml
├── QWEN.md
├── .gitignore
├── .env
├── .sqlx/ # SQLx编译时验证缓存
│ └── query-*.json
├── src/ # 源代码
├── migrations/ # 数据库迁移文件
├── tests/ # 集成测试

## src/ 源代码目录
src/
├── main.rs # 应用入口
├── lib.rs # 库定义
├── config.rs # 配置管理
├── error.rs # 错误类型定义
├── models.rs # 共享数据模型
│
├── api.rs # REST API路由模块定义
├── api/ # REST API路由实现
│ ├── auth.rs # 认证相关API
│ ├── game.rs # 游戏管理API
│ ├── player.rs # 玩家API
│ ├── admin.rs # 管理员API
│ └── director.rs # 导演API
│
├── websocket.rs # WebSocket模块定义
├── websocket/ # WebSocket处理实现
│ ├── handler.rs # 连接处理器
│ ├── player.rs # 玩家消息处理
│ ├── director.rs # 导演消息处理
│ └── broadcast.rs # 状态广播
│
├── services.rs # 业务服务模块定义
├── services/ # 业务服务实现
│ ├── auth_service.rs # 认证服务
│ ├── game_service.rs # 游戏管理服务
│ ├── player_service.rs # 玩家服务
│ └── game_logic.rs # 游戏核心逻辑
│
├── game_engine.rs # 游戏引擎模块定义
├── game_engine/ # 游戏引擎实现
│ ├── state.rs # 游戏状态管理
│ ├── action.rs # 行动处理
│ ├── items.rs # 道具系统
│ ├── safezone.rs # 安全区逻辑
│ └── voting.rs # 投票系统
│
├── utils.rs # 工具模块定义
└── utils/ # 工具函数实现
  ├──file.rs # 文件操作
  ├──validation.rs # 数据验证
  └──crypto.rs # 加密工具

## migrations/ 数据库迁移目录
migrations/
├── YYYYMMDDHHMMSS_description.up.sql
└── README.md

## tests/ 集成测试目录
tests/
├── api.rs
├── api/
│ ├── auth_test.rs # 认证API测试
│ ├── game_test.rs # 游戏API测试
│ └── player_test.rs # 玩家API测试
├── services.rs
├── services/
│ ├── game_service_test.rs
│ └── player_service_test.rs
├── game_engine.rs
└── game_engine/
├── action_test.rs
└── safezone_test.rs