## REST API

### 基础信息
- Base URL: `/api`
- 所有 API 响应格式: `application/json`
- 所有时间字段使用 ISO 8601 格式

### 状态码

| 状态码 | 说明                 |
|--------|----------------------|
| 200    | 请求成功             |
| 400    | 请求参数错误         |
| 401    | 未认证               |
| 403    | 权限不足             |
| 404    | 资源不存在           |
| 500    | 服务器内部错误       |

### 接口列表

#### 1. 管理员登录验证
```
POST /api/admin/login
```

**请求参数:**
```json
{
  "username": "string",  // 管理员用户名
  "password": "string"   // 管理员密码
}
```

**响应:**
```json
{
  "success": true,
  "token": "string",     // 认证令牌
  "expires_in": "integer" // 令牌过期时间（秒）
}
```

#### 2. 获取游戏列表
```
GET /api/games
```

**响应:**
```json
{
  "games": [
    {
      "id": "string",
      "name": "string",
      "description": "string",
      "status": "waiting|running|paused|ended",
      "phase": "day|night",
      "player_count": "integer",
      "max_players": 100,
      "start_time": "ISO8601 datetime",
      "end_time": "ISO8601 datetime"
    }
  ]
}
```

#### 3. 获取游戏基本信息
```
GET /api/game/{game_id}
```

**路径参数:**
- `game_id`: 游戏ID

**响应:**
```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "status": "waiting|running|paused|ended",
  "phase": "day|night",
  "start_time": "ISO8601 datetime",
  "end_time": "ISO8601 datetime",
  "player_count": "integer",
  "max_players": 100,
  "action_start_time": "ISO8601 datetime",
  "action_end_time": "ISO8601 datetime",
  "safe_zones": ["string"],
  "weather": "number",
  "announcements": ["string"]
}
```

#### 4. 获取玩家详细信息
```
GET /api/game/{game_id}/player/{player_id}?password=<password>
```

**路径参数:**
- `game_id`: 游戏ID
- `player_id`: 玩家ID

**查询参数:**
- `password`: 玩家密码

**响应:**
```json
{
  "id": "string",
  "name": "string",
  "life": "integer",        // 生命值 (0-100)
  "strength": "integer",    // 体力值 (0-100)
  "location": "string",     // 当前位置
  "things": ["string"],     // 拥有的道具列表
  "hands": ["string"],      // 装备的道具列表
  "able": "boolean",        // 是否可行动
  "injured": "integer",     // 是否受伤 (持续伤害标记)
  "vote": "integer",        // 持有的票数
  "ts": "number",           // 上次搜索时间戳
  "deliver": "integer",     // 传音次数标记
  "rest": "integer"         // 静养模式标记
}
```

#### 5. 获取游戏规则
```
GET /api/game/{game_id}/rules
```

**路径参数:**
- `game_id`: 游戏ID

**响应:**
```json
{
  "game_flow": {
    "day_duration": "integer",    // 白天时长(秒) *(默认值，导演可设置)*
    "night_duration": "integer"   // 夜晚时长(秒) *(默认值，导演可设置)*
  },
  "map": {
    "places": ["string"],         // 地点列表 *(默认列表，导演可设置)*
    "max_places": 50              // 最大地点数
  },
  "player": {
    "max_life": "integer",        // 最大生命值 *(默认值，导演可设置)*
    "max_strength": "integer",    // 最大体力值 *(默认值，导演可设置)*
    "daily_strength_recovery": "integer"  // 每日体力恢复值 *(默认值，导演可设置)*
  },
  "action": {
    "move_cost": "integer",       // 移动消耗体力 *(默认值，导演可设置)*
    "search_cost": "integer",     // 搜索消耗体力 *(默认值，导演可设置)*
    "search_cooldown": "integer"  // 搜索冷却时间(秒) *(默认值，导演可设置)*
  },
  "rest_mode": {
    "life_recovery": "integer",   // 静养模式生命恢复值 *(默认值，导演可设置)*
    "max_moves": "integer"        // 静养模式最大移动次数 *(默认值，导演可设置)*
  }
}
```

#### 6. 创建游戏
```
POST /api/admin/games
```

**请求参数:**
```json
{
  "name": "string",
  "description": "string",
  "director_password": "string",
  "max_players": "integer"
}
```

**响应:**
```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "director_password": "string",
  "max_players": "integer"
}
```

#### 7. 修改游戏设置
```
PUT /api/admin/game/{game_id}
```

**路径参数:**
- `game_id`: 游戏ID

**请求参数:**
```json
{
  "name": "string",           // 可选
  "description": "string",    // 可选
  "director_password": "string",  // 可选
  "max_players": "integer"    // 可选
}
```

**响应:**
```json
{
  "id": "string",
  "name": "string",
  "description": "string",
  "director_password": "string",
  "max_players": "integer"
}
```

#### 8. 删除游戏
```
DELETE /api/admin/game/{game_id}
```

**路径参数:**
- `game_id`: 游戏ID

**响应:**
```json
{
  "success": true,
  "message": "Game deleted successfully"
}
```

#### 9. 添加演员账户
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
  "player_name": "string",
  "password": "string"  // 6-8位字母数字
}
```

**响应:**
```json
{
  "id": "string",
  "name": "string",
  "password": "string"
}
```

#### 10. 获取演员列表
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

#### 11. 删除演员账户
```
DELETE /api/game/{game_id}/player/{player_id}?password=<director_password>
```

**路径参数:**
- `game_id`: 游戏ID
- `player_id`: 玩家ID

**查询参数:**
- `password`: 导演密码

**响应:**
```json
{
  "success": true,
  "message": "Player deleted successfully"
}
```

#### 12. 更新游戏规则配置
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

#### 13. 获取游戏日志
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

#### 14. 获取游戏统计
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

#### 15. 获取游戏完整状态快照
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

#### 16. 验证WebSocket连接凭据
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

#### 17. 获取地点状态
```
GET /api/game/{game_id}/places?password=<password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 导演或玩家密码

**响应:**
```json
{
  "places": [
    {
      "name": "string",
      "able": "boolean",      // 是否可用
      "exists": ["string"],   // 在该地点的角色和道具
      "safe": "boolean"       // 是否为安全区
    }
  ]
}
```

#### 18. 投票
```
POST /api/game/{game_id}/vote?password=<password>
```

**路径参数:**
- `game_id`: 游戏ID

**查询参数:**
- `password`: 玩家密码

**请求参数:**
```json
{
  "target": "string"  // 投票目标玩家名称
}
```

**响应:**
```json
{
  "success": true,
  "message": "Vote cast successfully"
}
```

#### 19. 获取投票结果
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
