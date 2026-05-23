# 错误处理

API 错误响应格式和常见错误代码说明。

## 错误响应格式

所有错误响应都遵循以下格式：

```json
{
  "success": false,
  "error": {
    "code": "string",      // 错误代码
    "message": "string",   // 错误描述
    "details": "object"    // 详细信息（可选）
  }
}
```

## 常见错误代码

### 认证相关错误
- `INVALID_CREDENTIALS`: 认证失败
- `GAME_NOT_FOUND`: 游戏不存在
- `PLAYER_NOT_FOUND`: 玩家不存在

### 操作限制错误
- `ACTION_NOT_ALLOWED`: 当前状态下不允许执行该操作
- `INSUFFICIENT_STRENGTH`: 体力不足
- `INVALID_TARGET`: 无效目标

### 游戏规则错误
- `SEARCH_COOLDOWN`: 搜索冷却中
- `VOTE_NOT_AVAILABLE`: 当前阶段不能投票
- `ALREADY_VOTED`: 已经投过票

### 系统错误
- `INTERNAL_ERROR`: 服务器内部错误
