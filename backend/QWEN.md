# Royale Arena 后端上下文

## 技术栈

- 编程语言：Rust
- Web框架：Axum（支持WebSocket）
- 数据库：SQLx库 + MySQL
- 序列化：serde库
- 日志：tracing + tower-http TraceLayer
- 测试：tokio-test + sqlx::test
- 其他：使用dotenvy加载环境变量

## 数据存储策略

### MySQL数据库
存储静态数据：
- 玩家账户信息（用户名、明文密码）
- 游戏基本信息（ID、名称、导演明文密码等）
- 游戏规则信息（JSON格式字段）
- 管理员帐户信息

### 内存存储
- 玩家实时状态
- 游戏进度信息
- 道具和位置信息

## 系统逻辑

1. REST API仅处理认证和状态查询
2. WebSocket仅处理实时操作指令
3. API层接收HTTP请求，Service层处理业务逻辑直接操作数据库（SQLx）
4. 游戏静态全局信息存储到Mysql数据库中
5. 游戏运行时数据存储到内存中，行动结束时持久化到文件

## 安全考虑

1. 除管理员密码外，所有密码明文存储
2. 通过URI中的密码参数直接访问导演控制台和玩家界面
3. WebSocket连接的身份验证机制

## 开发规范

### 模块系统

使用现代Rust模块系统：
- 不要创建mod.rs文件
- 创建与目录同名的.rs文件来定义模块
- 例如：src/user.rs 而不是 src/user/mod.rs

### 测试策略

测试模块分为以下四种：

1. 单元测试：纯逻辑测试，无数据库依赖
2. 数据库集成测试：每种主要服务有一个集成测试
   - 使用 #[sqlx::test(migrations = "./migrations")]
   - 测试真实数据库交互
3. API集成测试: 复杂业务流程的端到端测试
4. 测试数据自动管理，依赖sqlx::test事务自动清理

## 其他文档
- [规则存储设计文档](../docs/backend/rule-storage-design.md)
游戏规则以JSON格式存储在`rule_templates`表，`rules_config`字段存储完整配置
- [数据库迁移指南](../docs/backend/database-migration-workflow.md)
- [目录结构文档](../docs/backend/DIRECTORY.md)
- [任务列表](../docs/backend/task-list.md)