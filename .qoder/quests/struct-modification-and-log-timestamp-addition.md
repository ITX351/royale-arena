# ActionResult结构体修改与日志时间戳添加设计文档

## 1. 概述

本文档描述了对`ActionResult`结构体的修改方案，包括使`log_message`字段成为必需字段、添加时间戳字段，并调整其在WebSocket服务中的使用方式。同时，修改日志服务以使用ActionResult中生成的时间戳，而不是在创建日志时重新获取当前时间。

## 2. 架构设计

### 2.1 结构体修改

修改`ActionResult`结构体，添加时间戳字段并使`log_message`成为必需字段：

```rust
/// 动作处理结果，包含广播信息
#[derive(Debug, Clone)]
pub struct ActionResult {
    /// 动作处理结果数据
    pub data: serde_json::Value,
    /// 需要广播消息的玩家ID列表（包括发起者本人）
    pub broadcast_players: Vec<String>,
    /// 日志消息（必须提供）
    pub log_message: String,
    /// 消息类型
    pub message_type: MessageType,
    /// 动作处理时间戳
    pub timestamp: DateTime<Utc>,
}
```

### 2.2 工厂方法修改

修改`ActionResult`的工厂方法，确保在创建时生成时间戳：

```rust
impl ActionResult {
    /// 创建新的动作处理结果
    fn new(data: serde_json::Value, broadcast_players: Vec<String>, log_message: String, log_type: MessageType) -> Self {
        Self {
            data,
            broadcast_players,
            log_message,
            message_type: log_type,
            timestamp: Utc::now(),
        }
    }
    
    /// 创建新的动作处理结果（带系统日志消息）
    pub fn new_system_message(data: serde_json::Value, broadcast_players: Vec<String>, log_message: String) -> Self {
        ActionResult::new(data, broadcast_players, log_message, MessageType::SystemNotice)
    }
    
    /// 创建新的动作处理结果（带用户定向日志消息）
    pub fn new_user_message(data: serde_json::Value, broadcast_players: Vec<String>, log_message: String) -> Self {
        ActionResult::new(data, broadcast_players, log_message, MessageType::UserDirected)
    }
    
    /// 创建用于返回给前端的数据结构，排除`broadcast_players`字段
    pub fn to_client_response(&self) -> serde_json::Value {
        serde_json::json!({
            "data": self.data,
            "log_message": self.log_message,
            "message_type": self.message_type,
            "timestamp": self.timestamp
        })
    }
}
```

### 2.3 WebSocket服务调整

在WebSocket服务中，调整返回给前端的数据结构，使用ActionResult的新方法：

```rust
// 在处理玩家行动和导演控制的方法中，修改返回给前端的数据结构
let response_data = action_result.to_client_response();
```

### 2.4 日志服务调整

修改日志服务的`create_log`方法，接收时间戳参数而不是生成新的时间戳：

```rust
/// 创建游戏日志
pub async fn create_log(
    &self,
    game_id: &str,
    player_id: &str,
    message: &str,
    message_type: MessageType,
    timestamp: DateTime<Utc>,  // 添加时间戳参数
) -> Result<MessageRecord, String> {
    let id = Uuid::new_v4().to_string();
    
    // 根据消息类型确定数据库中的类型字符串
    let type_string = message_type.as_str();

    let result = sqlx::query!(
        r#"
        INSERT INTO game_logs (id, game_id, type, message, player_id, timestamp)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        id,
        game_id,
        type_string,
        message,
        player_id,
        timestamp  // 使用传入的时间戳
    )
    .execute(&self.pool)
    .await
    .map_err(|e| format!("Failed to create log: {}", e))?;

    if result.rows_affected() == 0 {
        return Err("Failed to insert log record".to_string());
    }

    Ok(MessageRecord {
        id,
        game_id: game_id.to_string(),
        message_type,
        message: message.to_string(),
        player_id: player_id.to_string(),
        timestamp,  // 使用传入的时间戳
    })
}
```

### 2.5 WebSocket服务中调用日志服务的调整

在WebSocket服务中调用日志服务时，传递ActionResult中的时间戳：

```rust
// 在处理玩家行动和导演控制的方法中，修改日志创建调用
let log_result = self.app_state.game_log_service.create_log(
    game_id,
    broadcast_player_id,
    &action_result.log_message,
    action_result.message_type.clone(),
    action_result.timestamp,  // 传递ActionResult中的时间戳
).await;
```

## 3. 数据模型

### 3.1 ActionResult结构体

| 字段名 | 类型 | 必需 | 描述 |
|--------|------|------|------|
| data | serde_json::Value | 是 | 动作处理结果数据 |
| broadcast_players | Vec<String> | 是 | 需要广播消息的玩家ID列表 |
| log_message | String | 是 | 日志消息 |
| message_type | MessageType | 是 | 消息类型 |
| timestamp | DateTime<Utc> | 是 | 动作处理时间戳 |

## 4. 业务逻辑层

### 4.1 ActionResult创建

在所有创建`ActionResult`的地方，使用新的工厂方法确保时间戳被正确生成：

1. 玩家行动处理方法（`game_state_player_actions.rs`）
2. 导演控制处理方法（`game_state_director_actions.rs`）

### 4.2 日志记录

在WebSocket服务中，当需要记录日志时，传递ActionResult中的时间戳给日志服务，确保日志时间与动作处理时间一致。