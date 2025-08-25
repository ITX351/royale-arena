# 玩家接口

玩家游戏相关的 API 接口，包括玩家状态查询、地点状态、投票等游戏内操作。

## 接口列表

### 1. 获取玩家详细信息
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

### 2. 获取地点状态
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

### 3. 投票
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
