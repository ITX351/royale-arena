# WebSocket广播系统重构设计文档

## 1. 概述

本设计文档旨在详细描述WebSocket广播系统的重构方案。当前的广播系统存在设计缺陷，特别是在消息封装和广播逻辑方面。重构后的系统将提供更清晰的接口，正确的隐私保护，以及更合理的广播机制。

## 2. 设计目标

1. 创建新的广播系统，替代旧的`old_broadcaster.rs`
2. 实现私有函数用于生成导演和玩家视角的消息体
3. 实现公有函数用于向导演和特定玩家广播消息
4. 确保玩家只能看到自己应看到的信息，保护隐私
5. 确保导演能看到完整的游戏状态
6. 正确处理`ActionResult`中的消息类型和内容

## 3. 系统架构

### 3.1 组件结构

```rust
// 新的广播器结构
pub struct MessageBroadcaster {
    connection_manager: ConnectionManager,
}

impl MessageBroadcaster {
    // 私有函数 - 生成导演视角消息
    fn generate_director_message(&self, game_state: &GameState, action_result: &ActionResult) -> serde_json::Value;
    
    // 私有函数 - 生成玩家视角消息
    fn generate_player_message(&self, game_state: &GameState, player: &Player, action_result: &ActionResult) -> serde_json::Value;
    
    // 公有函数 - 向所有导演广播
    pub async fn broadcast_to_directors(&self, game_state: &GameState, action_result: &ActionResult) -> Result<(), String>;
    
    // 公有函数 - 向特定玩家广播
    pub async fn broadcast_to_players(&self, game_state: &GameState, player_ids: &[String], action_result: &ActionResult) -> Result<(), String>;
}
```

### 3.2 消息结构

#### 3.2.1 导演视角消息体
导演可以看到完整的游戏状态，包括所有玩家的详细信息和所有地点的完整信息。

```json
{
  "type": "game_state_update",
  "data": {
    "game_phase": "day",
    "weather": 1.0,
    "night_start_time": "...",
    "night_end_time": "...",
    "next_night_destroyed_places": ["place1", "place2"],
    "players": {
      "player1": {
        "id": "player1",
        "name": "Player One",
        "location": "place1",
        "life": 100,
        "strength": 100,
        "inventory": [...],
        "equipped_item": "...",
        "hand_item": "...",
        "is_alive": true,
        "is_bound": false,
        "rest_mode": false
      }
    },
    "places": {
      "place1": {
        "name": "Place One",
        "players": ["player1"],
        "items": [...],
        "is_destroyed": false
      }
    },
    "action_result": {
      // ActionResult的内容
    }
  }
}
```

#### 3.2.2 玩家视角消息体
玩家只能看到自己的状态和地点的公共信息（是否被摧毁），但看不到其他玩家的详细信息和地点的物品信息。

```json
{
  "type": "game_state_update",
  "data": {
    "game_phase": "day",
    "weather": 1.0,
    "night_start_time": "...",
    "night_end_time": "...",
    "next_night_destroyed_places": ["place1", "place2"],
    "player": {
      "id": "player1",
      "name": "Player One",
      "location": "place1",
      "life": 100,
      "strength": 100,
      "is_alive": true,
      "rest_mode": false
    },
    "places": [
      {
        "name": "Place One",
        "is_destroyed": false
      }
    ],
    "action_result": {
      // ActionResult的内容
    }
  }
}
```

## 4. 接口设计

### 4.1 私有函数

#### 4.1.1 `generate_director_message`
```rust
fn generate_director_message(&self, game_state: &GameState, action_result: &ActionResult) -> serde_json::Value
```
- **功能**: 根据游戏状态和动作结果生成导演视角的完整消息体
- **参数**: 
  - `game_state`: 当前完整的游戏状态
  - `action_result`: 动作执行结果
- **返回**: 包含完整游戏状态和动作结果的JSON消息体

#### 4.1.2 `generate_player_message`
```rust
fn generate_player_message(&self, game_state: &GameState, player: &Player, action_result: &ActionResult) -> serde_json::Value
```
- **功能**: 根据游戏状态、玩家信息和动作结果生成玩家视角的消息体
- **参数**: 
  - `game_state`: 当前完整的游戏状态
  - `player`: 目标玩家的信息
  - `action_result`: 动作执行结果
- **返回**: 仅包含玩家可访问信息的JSON消息体

### 4.2 公有函数

#### 4.2.1 `broadcast_to_directors`
```rust
pub async fn broadcast_to_directors(&self, game_state: &GameState, action_result: &ActionResult) -> Result<(), String>
```
- **功能**: 向所有导演连接广播消息
- **参数**: 
  - `game_state`: 当前完整的游戏状态
  - `action_result`: 动作执行结果
- **返回**: 操作结果

#### 4.2.2 `broadcast_to_players`
```rust
pub async fn broadcast_to_players(&self, game_state: &GameState, player_ids: &[String], action_result: &ActionResult) -> Result<(), String>
```
- **功能**: 向指定的玩家列表广播消息，为每个玩家生成独立的隐私保护消息体
- **参数**: 
  - `game_state`: 当前完整的游戏状态
  - `player_ids`: 需要广播消息的玩家ID列表
  - `action_result`: 动作执行结果
- **返回**: 操作结果

## 5. 实现细节

### 5.1 隐私保护机制
- 玩家视角消息中不包含其他玩家的详细信息
- 玩家视角消息中的地点信息不包含物品和玩家列表
- 导演视角消息包含完整的游戏状态信息

### 5.2 ActionResult处理
- 直接将`ActionResult`附加到生成的消息体中
- 不再使用多层嵌套的`message_type`
- 保留`ActionResult`中的`message_type`字段

### 5.3 连接管理
- 使用现有的`ConnectionManager`进行实际的消息发送
- 通过连接类型区分导演和玩家连接

## 6. 服务层集成

在`websocket/service.rs`中，将使用新的广播器替换旧的广播逻辑：

```rust
// 旧逻辑（存在问题）
let _ = self.message_broadcaster.broadcast_system_message_to_directors(
    &serde_json::to_string(&response_json).unwrap_or_default()
).await;

// 新逻辑
let _ = self.message_broadcaster.broadcast_to_directors(&updated_game_state, &action_result).await;
```