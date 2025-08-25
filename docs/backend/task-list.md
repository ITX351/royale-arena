# Royale Arena 后端任务清单

## 1. 项目重构初始化
- [ ] 初始化新Rust项目 (Axum + SQLx)
- [ ] 配置SQLx CLI和迁移系统
- [ ] 建立数据库迁移目录结构
- [ ] 集成tracing和tower-http日志
- [ ] 配置开发环境和测试环境

## 2. 数据库设计与迁移
### 核心表结构迁移
- [ ] 创建管理员用户表迁移 (admin_users)
- [ ] 创建游戏信息表迁移 (games)
- [ ] 创建玩家信息表迁移 (players)
- [ ] 创建游戏规则模板表迁移 (rule_templates)
- [ ] 创建游戏状态表迁移 (game_states)
- [ ] 运行所有迁移创建表结构

## 3. 数据访问层开发
### SQLx数据库集成
- [ ] 实现SQLx数据库连接池
- [ ] 实现基础CRUD操作
- [ ] 实现事务管理
- [ ] 编写数据访问层集成测试

## 4. 核心业务模块开发
### 游戏管理模块
- [ ] 实现游戏创建、更新、删除服务
- [ ] 实现游戏规则模板管理服务
- [ ] 编写游戏管理集成测试

### 玩家管理模块
- [ ] 实现玩家账户管理服务
- [ ] 实现玩家状态管理服务
- [ ] 编写玩家管理集成测试

### 游戏逻辑引擎
- [ ] 实现行动时间窗口管理
- [ ] 实现玩家状态更新（生命值、体力值、位置等）
- [ ] 实现道具系统处理（搜索、使用、合成等）
- [ ] 实现安全区缩圈逻辑
- [ ] 实现投票系统处理
- [ ] 编写游戏逻辑单元测试

## 5. REST API开发 (Axum)
### 认证接口
- [ ] 实现`POST /api/admin/login` - 管理员登录验证
- [ ] 实现`POST /api/game/{game_id}/ws-auth` - WebSocket连接验证

### 游戏管理接口
- [ ] 实现`GET /api/games` - 获取游戏列表
- [ ] 实现`GET /api/game/{game_id}` - 获取游戏基本信息
- [ ] 实现`GET /api/game/{game_id}/rules` - 获取游戏规则
- [ ] 实现`GET /api/rule-templates` - 获取游戏规则模版列表
- [ ] 实现`GET /api/rule-templates/{template_id}` - 获取游戏规则模版详情

### 玩家接口
- [ ] 实现`GET /api/game/{game_id}/player/{player_id}` - 获取玩家详细信息
- [ ] 实现`GET /api/game/{game_id}/places` - 获取地点状态
- [ ] 实现`POST /api/game/{game_id}/vote` - 投票

### 导演接口
- [ ] 实现`POST /api/game/{game_id}/players` - 批量添加演员账户
- [ ] 实现`GET /api/game/{game_id}/players` - 获取演员列表
- [ ] 实现`DELETE /api/game/{game_id}/players` - 批量删除演员账户
- [ ] 实现`PUT /api/game/{game_id}/rules` - 更新游戏规则配置
- [ ] 实现`GET /api/game/{game_id}/logs` - 获取游戏日志
- [ ] 实现`GET /api/game/{game_id}/snapshot` - 获取游戏完整状态快照
- [ ] 实现`GET /api/game/{game_id}/votes` - 获取投票结果
- [ ] 实现`POST /api/game/{game_id}/reset` - 重置游戏
- [ ] 实现`GET /api/game/{game_id}/export` - 导出游戏数据

### 管理员接口
- [ ] 实现`POST /api/admin/login` - 管理员登录验证
- [ ] 实现`POST /api/admin/games` - 创建游戏
- [ ] 实现`PUT /api/admin/game/{game_id}` - 修改游戏设置
- [ ] 实现`DELETE /api/admin/game/{game_id}` - 删除游戏
- [ ] 实现`POST /api/admin/rule-templates` - 创建游戏规则模版
- [ ] 实现`PUT /api/admin/rule-templates/{template_id}` - 修改游戏规则模版
- [ ] 实现`DELETE /api/admin/rule-templates/{template_id}` - 删除游戏规则模版

## 6. WebSocket服务器开发
- [ ] 实现WebSocket连接和身份验证
- [ ] 实现玩家行动指令处理（移动、搜索、攻击、使用道具）
- [ ] 实现实时游戏状态推送
- [ ] 实现导演控制指令处理（开始行动、结束行动、缩圈、空投等）

## 7. 数据管理模块
- [ ] 实现动态数据内存管理
- [ ] 实现数据持久化（行动结束后保存到本地文件）
- [ ] 实现游戏状态序列化和反序列化

## 8. 测试体系建立
### 单元测试
- [ ] 为所有业务逻辑编写单元测试
- [ ] 为工具函数编写单元测试

### 集成测试
- [ ] 为每个主要服务编写集成测试
- [ ] 使用`#[sqlx::test(migrations = "./migrations")]`
- [ ] 测试数据库交互和业务逻辑

### API测试
- [ ] 为复杂API流程编写端到端测试
- [ ] 测试WebSocket通信流程

## 9. 系统优化与部署
- [ ] 实现高并发处理能力
- [ ] 优化内存管理
- [ ] 配置Docker部署环境
- [ ] 配置日志收集和监控
- [ ] 进行性能测试和压力测试