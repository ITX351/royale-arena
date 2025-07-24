# 认证接口

WebSocket 连接认证相关的 API 接口，用于获取 WebSocket 连接所需的认证令牌。

## 接口列表

### 1. 验证WebSocket连接凭据
```
POST /api/game/{game_id}/ws-auth
```

**路径参数:**
- `game_id`: 游戏ID

**请求参数:**
```json
{
  "password": "string",
  "user_type": "player|director"
}
```

**响应:**
```json
{
  "success": true,
  "token": "string",  // WebSocket连接令牌
  "user_id": "string",
  "user_name": "string"
}
```

## 使用说明

1. 客户端首先调用此接口获取 WebSocket 连接令牌
2. 使用获取的令牌建立 WebSocket 连接
3. 令牌用于身份验证和权限控制
4. 不同用户类型（玩家/导演）将获得不同的权限级别
