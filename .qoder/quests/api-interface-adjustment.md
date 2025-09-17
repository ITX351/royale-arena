# API接口调整设计文档

## 1. 概述

本文档描述了对 Royale Arena 项目 API 接口的调整需求，主要包括两个任务：

1. 将 `src\game\service.rs` 中的 `get_player_messages` 方法移动到 `src\game\log_service.rs` 中
2. 创建一个新的 API 接口用于验证游戏密码，并修改前端逻辑以使用该新接口

所有输出使用简体中文。

## 2. 实现计划

### 2.1 后端实现
1. 在 `GameLogService` 中实现 `get_player_messages` 方法
2. 从 `GameService` 中移除 `get_player_messages` 方法
3. 在 `GameService` 中添加 `authenticate_game` 方法
4. 在 `game/handlers.rs` 中添加对应的处理函数
5. 在 `routes.rs` 中注册新的路由

### 2.2 前端实现
1. 在 `directorService.ts` 中添加 `authenticateGame` 方法
2. 更新 API 配置添加新的端点
3. 修改前端组件使用新的认证接口

## 2. 架构调整

### 2.1 服务层重构

将玩家消息获取功能从 `GameService` 移动到 `GameLogService` 中，以更好地分离关注点。

#### 2.1.1 当前实现
- `get_player_messages` 方法目前位于 `GameService` 中
- 该方法负责验证玩家身份并获取其消息记录

#### 2.1.2 调整后实现
- 将 `get_player_messages` 方法移动到 `GameLogService` 中
- `GameService` 将不再包含与日志直接相关的功能

### 2.2 新增认证API

新增一个简单的API用于验证游戏密码：

```
GET /api/game/{game_id}/auth?password=<director_password>
```

该API将执行以下验证逻辑：
1. 首先在演员表(actors)中验证是否有当前游戏的密码等于该密码，如果有则返回当前用户为演员
2. 然后检验game表中的导演密码是否与之相符，如果相符则返回当前用户为导演
3. 否则返回验证失败

### 2.2.1 实现细节
- 在 `game/handlers.rs` 中新增处理函数 `authenticate_game`
- 在 `routes.rs` 中注册新的路由端点
- 在 `GameService` 中新增认证方法实现具体逻辑

### 2.2.2 路由注册
在 `routes.rs` 中添加新的路由配置：

```rust
// 游戏认证路由
let auth_routes = Router::new()
    .route(&format!("{}/game/{{game_id}}/auth", api_prefix), get(authenticate_game))
    .with_state(app_state.clone());

// 合并路由
Router::new()
    .merge(public_routes)
    .merge(admin_routes)
    .merge(game_admin_routes)
    .merge(rule_template_admin_routes)
    .merge(director_routes)
    .merge(player_routes)
    .merge(auth_routes)  // 添加这一行
```

## 3. API设计

### 3.1 新增认证接口

#### 3.1.1 接口详情
- **路径**: `GET /api/game/{game_id}/auth`
- **方法**: GET
- **参数**:
  - 路径参数: `game_id` (string) - 游戏ID
  - 查询参数: `password` (string) - 明文密码
- **响应**:
  - 成功时返回包含用户角色信息的JSON对象
  - 失败时返回标准错误响应
- **认证**: 不需要JWT认证，通过密码验证身份

#### 3.1.2 响应格式

成功响应示例：
```json
"actor"
```

或

```json
"director"
```

错误响应示例：
```json
"invalid"
```

### 3.2 服务层方法调整

#### 3.2.1 GameLogService新增方法
在 `GameLogService` 中添加 `get_player_messages` 方法：

```rust
/// 获取玩家消息记录
pub async fn get_player_messages(&self, game_id: &str, player_id: &str, password: &str) -> Result<Vec<MessageRecord>, GameError> {
    // 验证请求参数
    let request = GetPlayerMessagesRequest {
        password: password.to_string(),
    };
    request.validate().map_err(GameError::ValidationError)?;
    
    // 验证玩家是否存在且密码正确
    let actor = sqlx::query!(
        "SELECT id FROM actors WHERE id = ? AND game_id = ? AND password = ?",
        player_id,
        game_id,
        password
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(GameError::DatabaseError)?;
    
    if actor.is_none() {
        return Err(GameError::ValidationError("Invalid player credentials".to_string()));
    }
    
    // 查询玩家相关的消息记录
    let messages = sqlx::query_as!(
        MessageRecord,
        r#"
        SELECT id, game_id, type as "message_type: MessageType", message, player_id, timestamp
        FROM game_logs 
        WHERE game_id = ? AND player_id = ?
        ORDER BY timestamp ASC
        "#,
        game_id,
        player_id
    )
    .fetch_all(&self.pool)
    .await
    .map_err(GameError::DatabaseError)?;
    
    Ok(messages)
}
```

### 3.2.2 新增认证方法
在 `GameService` 中添加游戏密码认证方法：

```rust
/// 游戏身份验证
pub async fn authenticate_game(&self, game_id: &str, password: &str) -> Result<String, GameError> {
    // 首先检查是否是演员密码
    let actor = sqlx::query!(
        "SELECT id FROM actors WHERE game_id = ? AND password = ?",
        game_id,
        password
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(GameError::DatabaseError)?;
    
    if actor.is_some() {
        return Ok("actor".to_string());
    }
    
    // 然后检查是否是导演密码
    let game = sqlx::query!(
        "SELECT id, director_password FROM games WHERE id = ?",
        game_id
    )
    .fetch_optional(&self.pool)
    .await
    .map_err(GameError::DatabaseError)?;
    
    match game {
        Some(game) if game.director_password == password => {
            Ok("director".to_string())
        },
        _ => Ok("invalid".to_string())
    }
}
```

#### 3.2.3 GameService移除方法
从 `GameService` 中移除 `get_player_messages` 方法。

### 3.2.4 新增处理函数
在 `game/handlers.rs` 中添加新的处理函数：

```rust
/// 游戏身份验证处理函数
pub async fn authenticate_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<String>, GameError> {
    let password = params.get("password").ok_or_else(|| {
        GameError::ValidationError("Password is required".to_string())
    })?;
    
    let result = state.game_service.authenticate_game(&game_id, password).await?;
    
    Ok(Json(result))
}
```

## 4. 前端调整

### 4.1 新增认证服务方法

在前端服务层添加新的认证方法：

```typescript
// directorService.ts
async authenticateGame(
  gameId: string,
  password: string
): Promise<string> {
  const response = await apiClient.get(`/game/${gameId}/auth`, {
    params: { password }
  });
  return response.data;
}
```

### 4.2 修改前端逻辑

修改前端使用新API进行身份验证的逻辑，替换原有的验证方式。

### 4.3 API路由配置

在 `frontend/src/services/config.ts` 中添加新的API端点配置：

```typescript
// API endpoints
export const API_ENDPOINTS = {
  // ... existing endpoints ...
  
  // 游戏认证相关
  GAME_AUTH: (gameId: string) => `/game/${gameId}/auth`
}
```

## 5. 数据模型

### 5.1 认证响应模型

认证结果为简单的字符串，可能的值为：
- "actor"：表示验证结果为演员
- "director"：表示验证结果为导演
- "invalid"：表示验证失败