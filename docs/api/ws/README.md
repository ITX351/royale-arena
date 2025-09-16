# WebSocket 接口文档

本文档描述了 Royale Arena 游戏的 WebSocket 接口，用于实时游戏状态更新和玩家行动指令。

## 基础信息
- WebSocket URL: `ws://localhost:PORT/ws`
- 消息格式: JSON
- 编码: UTF-8

## 连接参数
连接时需要提供身份认证信息:
- `game_id`: 游戏ID
- `password`: 导演或玩家密码
- `user_type`: "player" 或 "director"

## 连接示例
```javascript
const ws = new WebSocket('ws://localhost:8080/ws?game_id=123&password=secret&user_type=player');
```

## 消息格式
所有 WebSocket 消息都使用 JSON 格式:
```json
{
  "type": "string",     // 消息类型
  "data": "object"      // 消息数据
}
```

## 消息类型

### 客户端发送消息
| 类型 | 说明 | 发送者 |
|------|------|--------|
| player_action | 玩家行动指令 | 玩家 |
| director_action | 导演控制指令 | 导演 |

### 服务端推送消息
| 类型 | 说明 | 接收者 |
|------|------|--------|
| game_state | 游戏状态更新 | 所有连接者 |
| player_update | 玩家状态更新 | 对应玩家 |
| system_message | 系统消息 | 所有连接者 |
| error | 错误信息 | 对应连接者 |

## 文档结构

- [基础接口](README.md) - WebSocket 基础信息和连接规范
- [玩家行动指令](player-actions.md) - 玩家可执行的游戏行动
- [导演控制指令](director-actions.md) - 导演游戏管理和控制功能

## 权限说明

- **导演**: 可发送所有导演控制指令，接收完整游戏状态
- **玩家**: 只能发送玩家行动指令，接收受限的游戏状态

## 架构说明

WebSocket服务采用分离式架构设计：
- WebSocket服务负责处理连接和消息传输
- 游戏状态由专门的游戏状态管理器统一管理
- WebSocket服务通过游戏服务访问和更新游戏状态
- 游戏状态支持内存存储和磁盘持久化

这种架构确保了：
1. 游戏状态管理与WebSocket连接处理完全分离
2. 多个WebSocket连接共享同一套游戏状态
3. 游戏状态的持久化和恢复机制统一管理
4. 便于扩展和维护

新的架构设计解决了之前的问题：
- 不再为每个WebSocket连接创建独立的游戏系统
- 游戏状态统一由游戏服务管理
- WebSocket服务仅负责通信，不直接管理游戏状态
- 通过游戏服务的GameStateManager组件访问和更新游戏状态

## 错误处理

连接错误或消息格式错误时，服务端会发送错误消息：
```json
{
  "type": "error",
  "data": {
    "code": "ERROR_CODE",
    "message": "错误描述"
  }
}
```

更多详细信息请参考各个分类文档。