# 后端WebSocket消息体优化设计文档

## 1. 概述

本设计文档旨在优化后端WebSocket返回的JSON消息体实现，主要目标包括：

1. 实现一个统一的消息生成函数模块，所有消息发送前都要经过该模块处理
2. 简化和统一现有代码中分散的消息生成逻辑
3. 修正`game_state_director_actions.rs`和`game_state_player_actions.rs`中错误的消息结构嵌套

通过本次优化，可以提高代码的可维护性、一致性和可读性。

## 2. 当前问题分析

### 2.1 消息生成逻辑分散

当前在`websocket/service.rs`中存在多个直接使用`serde_json::json!`宏生成消息的地方，导致消息格式不统一，维护困难。

### 2.2 消息结构嵌套错误

在`game_state_director_actions.rs`和`game_state_player_actions.rs`中，每个动作生成的`response`变量都自作主张地添加了`type`和`data`字段，但实际上这些消息在发送给客户端之前还需要额外的包装层，导致结构冗余。

### 2.3 消息生成方式不统一

不同地方使用不同的方式生成消息，缺乏统一的接口和规范。

## 3. 设计方案

### 3.1 创建统一消息生成模块

创建一个新的模块`message_formatter.rs`，用于统一处理所有WebSocket消息的生成。

#### 3.1.1 内部函数设计

```rust
/// 内部消息生成函数
fn generate_message(message_type: &str, data: serde_json::Value) -> serde_json::Value {
    json!({
        "type": message_type,
        "data": data
    })
}
```

#### 3.1.2 公共接口设计

提供三个公共函数用于生成不同类型的消息：

1. 系统消息生成函数
2. 游戏状态消息生成函数
3. 错误消息生成函数

### 3.2 修正动作处理响应结构

修改`game_state_director_actions.rs`和`game_state_player_actions.rs`中的所有动作处理函数，将`response`变量重命名为`data`，并移除其中的`type`字段和外层`data`包装。

### 3.3 统一消息发送接口

修改`websocket/service.rs`中所有使用`sender.send`和`socket.send`的地方，使用新创建的消息生成模块。

## 4. 实施步骤

### 4.1 创建消息格式化模块

由于工具限制，无法直接创建新文件。需要通过项目开发流程创建文件`backend/src/websocket/message_formatter.rs`，实现以下功能：

```rust
//! WebSocket消息格式化模块

use serde_json::{json, Value as JsonValue};
use axum::extract::ws::{Message, Utf8Bytes};

/// 内部消息生成函数
fn generate_message(message_type: &str, data: JsonValue) -> JsonValue {
    json!({
        "type": message_type,
        "data": data
    })
}

/// 生成系统消息
pub fn system_message(data: JsonValue) -> JsonValue {
    generate_message("system_message", data)
}

/// 生成游戏状态消息
pub fn game_state_message(data: JsonValue) -> JsonValue {
    generate_message("game_state", data)
}

/// 生成错误消息
pub fn error_message(data: JsonValue) -> JsonValue {
    generate_message("error", data)
}

/// 将JSON值转换为可发送的WebSocket消息
pub fn to_websocket_message(json_value: JsonValue) -> Message {
    Message::Text(Utf8Bytes::from(serde_json::to_string(&json_value).unwrap()))
}
```

### 4.2 修改动作处理函数

#### 4.2.1 修改导演动作处理函数

将`game_state_director_actions.rs`中的所有`response`变量改为`data`，并移除外层包装：

```rust
// 修改前
let response = serde_json::json!({
    "type": "game_state",
    "data": {
        "night_start_time": time
    }
});

// 修改后
let data = serde_json::json!({
    "night_start_time": time
});
```

需要修改的函数包括：
- `handle_set_night_start_time`
- `handle_set_night_end_time`
- `handle_modify_place`
- `handle_set_destroy_places`
- `handle_drop`
- `handle_weather`
- `handle_life`
- `handle_strength`
- `handle_move_player`
- `handle_give`
- `handle_rope_action`
- `handle_broadcast`
- `handle_director_message_to_player`

以`handle_set_night_start_time`函数为例，修改如下：

```rust
// 修改前
let response = serde_json::json!({
    "type": "game_state",
    "data": {
        "night_start_time": time
    }
});

// 修改后
let data = serde_json::json!({
    "night_start_time": time
});
```

#### 4.2.2 修改玩家动作处理函数

同样修改`game_state_player_actions.rs`中的所有动作处理函数，包括：
- `handle_born_action`
- `handle_move_action`
- `handle_search_action`
- `handle_pick_action`
- `handle_attack_action`
- `handle_equip_action`
- `handle_use_action`
- `handle_throw_action`
- `handle_deliver_action`
- `handle_send_action`

以`handle_born_action`函数为例，修改如下：

```rust
// 修改前
let response = serde_json::json!({
    "type": "player_update",
    "data": {
        "location": place_name
    }
});

// 修改后
let data = serde_json::json!({
    "location": place_name
});
```

### 4.3 更新服务模块中的消息发送

修改`websocket/service.rs`中所有直接使用JSON生成的地方，改用新的消息格式化模块。

需要更新的地方包括：
1. `handle_websocket_connection`函数中的连接成功消息
2. `authenticate_connection`函数中的认证失败消息
3. 其他直接使用`serde_json::json!`宏生成消息的地方

以`handle_websocket_connection`函数为例，修改如下：

```rust
// 修改前
let success_msg = json!({
    "type": "system_message",
    "data": {
        "message": "WebSocket connection established successfully"
    }
});

// 修改后
let success_msg_data = json!({
    "message": "WebSocket connection established successfully"
});
let success_msg = message_formatter::system_message(success_msg_data);
```

此外，还需要更新`ActionResult`的使用方式。在动作处理函数返回的`ActionResult`中，`data`字段应该只包含实际的数据内容，而不包含`type`和外层的`data`包装。

## 5. 影响范围

### 5.1 文件变更列表

1. 新增文件：
   - `backend/src/websocket/message_formatter.rs` - 消息格式化模块

2. 修改文件：
   - `backend/src/websocket/game_state_director_actions.rs` - 导演动作处理函数
   - `backend/src/websocket/game_state_player_actions.rs` - 玩家动作处理函数
   - `backend/src/websocket/service.rs` - WebSocket服务模块

3. 更新模块引用：
   - 在`backend/src/websocket.rs`中添加对`message_formatter`模块的引用
   ```rust
   pub mod message_formatter;
   ```

