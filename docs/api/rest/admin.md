# 管理员接口

系统管理员相关的 API 接口，用于管理游戏实例和系统级配置。

## 接口列表

### 1. 管理员登录验证
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


### 2. 创建游戏
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

### 3. 修改游戏设置
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

### 4. 删除游戏
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

### 5. 创建游戏规则模版
```
POST /api/admin/rule-templates
```

**请求参数:**
```json
{
  "template_name": "string",      // 模版名称
  "description": "string",        // 模版描述
  "game_flow": {
    "day_duration": "integer|null",    // 白天时长(秒)，null表示使用默认值
    "night_duration": "integer|null"   // 夜晚时长(秒)，null表示使用默认值
  },
  "map": {
    "places": ["string"]          // 地点列表，可选
  },
  "player": {
    "max_life": "integer|null",        // 最大生命值，null表示使用默认值
    "max_strength": "integer|null",    // 最大体力值，null表示使用默认值
    "daily_strength_recovery": "integer|null"  // 每日体力恢复值，null表示使用默认值
  },
  "action": {
    "move_cost": "integer|null",       // 移动消耗体力，null表示使用默认值
    "search_cost": "integer|null",     // 搜索消耗体力，null表示使用默认值
    "search_cooldown": "integer|null"  // 搜索冷却时间(秒)，null表示使用默认值
  },
  "rest_mode": {
    "life_recovery": "integer|null",   // 静养模式生命恢复值，null表示使用默认值
    "max_moves": "integer|null"        // 静养模式最大移动次数，null表示使用默认值
  }
}
```

**响应:**
```json
{
  "id": "string",
  "template_name": "string",
  "description": "string",
  "game_flow": {
    "day_duration": "integer|null",
    "night_duration": "integer|null"
  },
  "map": {
    "places": ["string"]
  },
  "player": {
    "max_life": "integer|null",
    "max_strength": "integer|null",
    "daily_strength_recovery": "integer|null"
  },
  "action": {
    "move_cost": "integer|null",
    "search_cost": "integer|null",
    "search_cooldown": "integer|null"
  },
  "rest_mode": {
    "life_recovery": "integer|null",
    "max_moves": "integer|null"
  },
  "created_at": "ISO8601 datetime",
  "updated_at": "ISO8601 datetime"
}
```

### 6. 修改游戏规则模版
```
PUT /api/admin/rule-templates/{template_id}
```

**路径参数:**
- `template_id`: 模版ID

**请求参数:**
```json
{
  "template_name": "string",      // 可选
  "description": "string",        // 可选
  "game_flow": {
    "day_duration": "integer|null",    // 可选
    "night_duration": "integer|null"   // 可选
  },
  "map": {
    "places": ["string"]          // 可选
  },
  "player": {
    "max_life": "integer|null",        // 可选
    "max_strength": "integer|null",    // 可选
    "daily_strength_recovery": "integer|null"  // 可选
  },
  "action": {
    "move_cost": "integer|null",       // 可选
    "search_cost": "integer|null",     // 可选
    "search_cooldown": "integer|null"  // 可选
  },
  "rest_mode": {
    "life_recovery": "integer|null",   // 可选
    "max_moves": "integer|null"        // 可选
  }
}
```

**响应:**
```json
{
  "id": "string",
  "template_name": "string",
  "description": "string",
  "game_flow": {
    "day_duration": "integer|null",
    "night_duration": "integer|null"
  },
  "map": {
    "places": ["string"]
  },
  "player": {
    "max_life": "integer|null",
    "max_strength": "integer|null",
    "daily_strength_recovery": "integer|null"
  },
  "action": {
    "move_cost": "integer|null",
    "search_cost": "integer|null",
    "search_cooldown": "integer|null"
  },
  "rest_mode": {
    "life_recovery": "integer|null",
    "max_moves": "integer|null"
  },
  "created_at": "ISO8601 datetime",
  "updated_at": "ISO8601 datetime"
}
```

### 7. 删除游戏规则模版
```
DELETE /api/admin/rule-templates/{template_id}
```

**路径参数:**
- `template_id`: 模版ID

**响应:**
```json
{
  "success": true,
  "message": "Rule template deleted successfully"
}
```
