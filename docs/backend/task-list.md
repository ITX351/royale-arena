# Royale Arena 后端任务清单

## 1. 项目初始化
- [x] 初始化Rust项目
- [x] 集成Web框架（Actix-web）
- [x] 集成WebSocket库
- [x] 添加MySQL驱动依赖项（尚未实际集成使用）
- [x] 集成serde用于JSON序列化
- [x] 集成tracing用于日志记录

## 2. 核心功能模块开发
### REST API服务器
- [ ] 实现基础服务器框架并解决Docker访问问题
- [ ] 实现用户认证（管理员登录验证）
- [ ] 实现游戏状态及设置查询
- [ ] 实现玩家信息获取
- [ ] 实现游戏规则获取

### WebSocket服务器
- [ ] 实现玩家行动指令接收
- [ ] 实现实时游戏状态推送
- [ ] 实现导演操作指令处理
- [ ] 实现行动结果反馈

### 游戏逻辑引擎
- [ ] 实现行动时间窗口管理
- [ ] 实现玩家状态更新（生命值、体力值、位置等）
- [ ] 实现道具系统处理（搜索、使用、合成等）
- [ ] 实现安全区缩圈逻辑
- [ ] 实现投票系统处理

### 数据管理模块
- [x] 实现静态数据访问（通过MySQL存储玩家信息等）
- [ ] 实现动态数据管理（游戏运行时所有数据在内存中处理）
- [ ] 实现数据持久化（行动结束后将游戏状态保存到本地文件）

## 3. MySQL数据库集成
### 数据库设计
- [x] 设计管理员用户表结构
- [ ] 设计游戏信息表结构
- [ ] 设计玩家信息表结构

### 数据库连接与测试
- [x] 实现MySQL数据库连接
- [x] 实现数据库连接池
- [x] 编写数据库连接测试
- [ ] 实现基本的CRUD操作

## 4. 接口实现
### REST API端点

#### 认证接口
- [ ] 实现`POST /api/admin/login` - 管理员登录验证
- [ ] 实现`POST /api/game/{game_id}/ws-auth` - 验证WebSocket连接凭据

#### 游戏管理接口
- [ ] 实现`GET /api/games` - 获取游戏列表
- [ ] 实现`GET /api/game/{game_id}` - 获取游戏基本信息
- [ ] 实现`GET /api/game/{game_id}/rules` - 获取游戏规则
- [ ] 实现`GET /api/rule-templates` - 获取游戏规则模版列表
- [ ] 实现`GET /api/rule-templates/{template_id}` - 获取游戏规则模版详情

#### 玩家接口
- [ ] 实现`GET /api/game/{game_id}/player/{player_id}` - 获取玩家详细信息
- [ ] 实现`GET /api/game/{game_id}/places` - 获取地点状态
- [ ] 实现`POST /api/game/{game_id}/vote` - 投票

#### 导演接口
- [ ] 实现`POST /api/game/{game_id}/players` - 批量添加演员账户
- [ ] 实现`GET /api/game/{game_id}/players` - 获取演员列表
- [ ] 实现`DELETE /api/game/{game_id}/players` - 批量删除演员账户
- [ ] 实现`PUT /api/game/{game_id}/rules` - 更新游戏规则配置
- [ ] 实现`GET /api/game/{game_id}/logs` - 获取游戏日志
- [ ] 实现`GET /api/game/{game_id}/stats` - 获取游戏统计
- [ ] 实现`GET /api/game/{game_id}/snapshot` - 获取游戏完整状态快照
- [ ] 实现`GET /api/game/{game_id}/votes` - 获取投票结果
- [ ] 实现`POST /api/game/{game_id}/reset` - 重置游戏
- [ ] 实现`GET /api/game/{game_id}/export` - 导出游戏数据

#### 管理员接口
- [ ] 实现`POST /api/admin/login` - 管理员登录验证
- [x] 实现`POST /api/admin/games` - 创建游戏
- [x] 实现`PUT /api/admin/game/{game_id}` - 修改游戏设置
- [x] 实现`DELETE /api/admin/game/{game_id}` - 删除游戏
- [x] 实现`POST /api/admin/rule-templates` - 创建游戏规则模版
- [x] 实现`PUT /api/admin/rule-templates/{template_id}` - 修改游戏规则模版
- [x] 实现`DELETE /api/admin/rule-templates/{template_id}` - 删除游戏规则模版

### WebSocket通信
- [ ] 实现连接时的身份验证
- [ ] 实现玩家行动指令处理（移动、搜索、攻击、使用道具）
- [ ] 实现服务器实时状态推送
- [ ] 实现导演控制指令处理（开始行动、结束行动、缩圈、空投等）

## 5. 系统约束与性能
- [ ] 确保不使用游戏引擎或第三方实时通信服务
- [ ] 确保不使用ORM框架，直接操作MySQL数据库
- [ ] 确保WebSocket仅处理实时操作指令
- [ ] 确保REST API仅处理认证和状态查询
- [ ] 确保游戏过程中不访问数据库，所有数据都在内存中
- [ ] 确保不复用旧版Python系统的任何代码
- [ ] 实现高并发处理能力，支持最多100名玩家同时在线
- [ ] 实现低延迟响应，确保实时操作的流畅性
- [ ] 实现内存高效管理，避免内存泄漏
- [ ] 确保系统稳定性，能够长时间运行

## 6. 安全与测试
- [x] 除管理员密码外，所有密码明文存储（仅为项目特定设计）
- [ ] 实现通过URI中的密码参数直接访问导演控制台和玩家界面
- [ ] 实现WebSocket连接的身份验证机制
- [ ] 实现防止恶意操作的验证机制
- [x] 编写单元测试和集成测试
- [ ] 进行性能测试和压力测试