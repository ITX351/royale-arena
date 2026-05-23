# 游戏管理接口

基础的游戏管理接口，包括游戏列表查询、游戏信息获取等公共接口。

## 接口列表

### 1. 获取游戏列表
```
GET /api/games
GET /api/admin/games
```

**说明**:
- `GET /api/games`: 公开接口，不显示导演密码
- `GET /api/admin/games`: 管理员接口，携带有效JWT token时显示导演密码

**查询参数:**
- `status`: 可选，筛选特定状态的游戏 (waiting|running|paused|ended)

**响应:**
```json
{
  "success": true,
  "data": [
    {
      "id": "string",
      "name": "string",
      "description": "string",
      "status": "waiting|running|paused|ended",
      "player_count": "integer",
      "max_players": "integer",
      "created_at": "ISO8601 datetime",
      "director_password": "string (仅通过/api/admin/games访问且携带有效JWT token时返回)"
    }
  ]
}
```

### 2. 获取指定游戏信息（包括规则）
```
GET /api/games/{game_id}
GET /api/admin/games/{game_id}
```

**说明**:
- `GET /api/games/{game_id}`: 公开接口，不显示导演密码
- `GET /api/admin/games/{game_id}`: 管理员接口，携带有效JWT token时显示导演密码

**路径参数:**
- `game_id`: 游戏ID

**响应:**
```json
{
  "success": true,
  "data": {
    "id": "string",
    "name": "string",
    "description": "string",
    "status": "waiting|running|paused|ended",
    "player_count": "integer",
    "max_players": "integer",
    "created_at": "ISO8601 datetime",
    "director_password": "string (仅通过/api/admin/games/{game_id}访问且携带有效JWT token时返回)",
    "rule_template": {
      "id": "string",
      "template_name": "string",
      "description": "string",
      "rules_config": {}
    } | null
  }
}
```
