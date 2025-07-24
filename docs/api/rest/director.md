# 导演接口

游戏导演专用的管理接口，用于管理玩家、游戏规则配置、游戏监控等。

## 接口列表

### 1. 批量添加演员账户
```
POST /api/game/{game_id}/players?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**请求参数:**
```json
{
  "players": [
    {
      "player_name": "string",
      "password": "string"  // 6-8位字母数字
    }
  ]
}
```

**响应:**
```json
{
  "success": [
    {
      "id": "string",
      "name": "string",
      "password": "string"
    }
  ],
  "failed": [
    {
      "player_name": "string",
      "reason": "string"
    }
  ]
}
```

### 2. 获取演员列表
```
GET /api/game/{game_id}/players?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**响应:**
```json
{
  "players": [
    {
      "id": "string",
      "name": "string"
      // 注意：出于安全考虑，不返回密码
    }
  ]
}
```

### 3. 批量删除演员账户
```
DELETE /api/game/{game_id}/players?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**请求参数:**
```json
{
  "player_ids": ["string"]
}
```

**响应:**
```json
{
  "success": [
    {
      "id": "string",
      "name": "string",
      "message": "Player deleted successfully"
    }
  ],
  "failed": [
    {
      "id": "string",
      "reason": "string"
    }
  ]
}
```

### 4. 更新游戏规则配置
```
PUT /api/game/{game_id}/rules?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**请求参数:**
```json
{
  "game_flow": {
    "day_duration": "integer",    // 可选
    "night_duration": "integer"   // 可选
  },
  "map": {
    "places": ["string"]          // 可选
  },
  "player": {
    "max_life": "integer",        // 可选
    "max_strength": "integer",    // 可选
    "daily_strength_recovery": "integer"  // 可选
  },
  "action": {
    "move_cost": "integer",       // 可选
    "search_cost": "integer",     // 可选
    "search_cooldown": "integer"  // 可选
  },
  "rest_mode": {
    "life_recovery": "integer",   // 可选
    "max_moves": "integer"        // 可选
  }
}
```

**响应:**
```json
{
  "success": true,
  "message": "Rules updated successfully"
}
```

### 5. 获取游戏日志
```
GET /api/game/{game_id}/logs?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码
- `limit`: integer (可选) - 返回记录数限制，默认50
- `offset`: integer (可选) - 偏移量，默认0

**响应:**
```json
{
  "logs": [
    {
      "timestamp": "ISO8601 datetime",
      "level": "info|warn|error",
      "message": "string",
      "player": "string"  // 可选，相关玩家
    }
  ]
}
```

### 6. 获取游戏统计
```
GET /api/game/{game_id}/stats?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**响应:**
```json
{
  "player_count": "integer",
  "alive_players": "integer",
  "total_actions": "integer",
  "start_time": "ISO8601 datetime",
  "duration": "integer",  // 游戏持续时间(秒)
  "votes_cast": "integer",
  "eliminations": "integer"
}
```

### 7. 获取游戏完整状态快照
```
GET /api/game/{game_id}/snapshot?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**响应:**
```json
{
  "timestamp": "ISO8601 datetime",
  "players": [],    // 玩家状态列表
  "places": [],     // 地点状态列表
  "game_state": {
    "phase": "day|night",
    "weather": "number",
    "safe_zones": ["string"],
    "votes": {}
  }
}
```

### 8. 获取投票结果
```
GET /api/game/{game_id}/votes?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**响应:**
```json
{
  "votes": {
    "player_name": "integer"  // 玩家名称对应票数
  },
  "total_votes": "integer",
  "most_voted": "string"      // 得票最多的玩家，平票时为null
}
```

### 9. 重置游戏
```
POST /api/game/{game_id}/reset?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**响应:**
```json
{
  "success": true,
  "message": "Game reset successfully"
}
```

### 10. 导出游戏数据
```
GET /api/game/{game_id}/export?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演密码

**响应:**
```json
{
  "game_info": {
    "id": "string",
    "name": "string",
    "description": "string",
    "start_time": "ISO8601 datetime",
    "end_time": "ISO8601 datetime",
    "duration": "integer"
  },
  "players": [
    {
      "id": "string",
      "name": "string",
      "final_status": "alive|dead",
      "death_cause": "string",
      "final_location": "string"
    }
  ],
  "logs": [
    {
      "timestamp": "ISO8601 datetime",
      "level": "info|warn|error",
      "message": "string",
      "player": "string"
    }
  ],
  "statistics": {
    "total_players": "integer",
    "final_survivors": "integer",
    "total_actions": "integer",
    "votes_cast": "integer",
    "eliminations": "integer"
  }
}
```
