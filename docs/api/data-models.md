# 数据模型

WebSocket 消息格式、数据结构和错误处理的完整定义。

## 服务端推送消息类型

### 1. 游戏状态更新
```json
{
  "type": "game_state_update",
  "data": {
    "players": [],     // 玩家状态列表
    "places": [],      // 地点状态列表
    "time_left": 0,    // 剩余时间(秒)
    "phase": "day|night",
    "weather": "number",
    "safe_zones": ["string"],
    "announcements": [] // 系统公告列表
  }
}
```

### 2. 行动结果反馈
```json
{
  "type": "action_result",
  "data": {
    "success": true,
    "message": "string",  // 操作结果描述
    "action": "string",   // 行动类型
    "effects": {}         // 行动效果详情
  }
}
```

### 3. 系统公告
```json
{
  "type": "announcement",
  "data": {
    "content": "string",  // 公告内容
    "level": "info|warn|error",
    "timestamp": "ISO8601 datetime"
  }
}
```

### 4. 投票状态更新
```json
{
  "type": "vote_update",
  "data": {
    "votes": {},          // 投票统计
    "time_left": "integer", // 投票剩余时间
    "can_vote": "boolean"   // 当前用户是否可投票
  }
}
```

### 5. 游戏阶段变更
```json
{
  "type": "phase_change",
  "data": {
    "new_phase": "day|night",
    "duration": "integer",
    "message": "string"
  }
}
```

### 6. 错误消息
```json
{
  "type": "error",
  "data": {
    "code": "string",    // 错误代码
    "message": "string"  // 错误描述
  }
}
```

## 数据模型

### 管理员账户
```json
{
  "id": "string",
  "username": "string",
  "password": "string"  // 密文存储
}
```

### 演员账户
```json
{
  "id": "string",
  "name": "string",
  "password": "string",  // 6-8位字母数字，明文存储（仅为项目特定设计）
  "team_id": "integer"  // 队伍ID，用于标识玩家所属队伍，0表示无队伍
}
```

### 玩家状态
```json
{
  "id": "string",
  "name": "string",
  "team_id": "integer",  // 队伍ID，用于标识玩家所属队伍，0表示无队伍
  "life": "integer",      // 生命值 (0-100)
  "strength": "integer",  // 体力值 (0-100)
  "location": "string",   // 当前位置
  "things": [],           // 拥有的道具
  "hands": [],            // 装备的道具
  "able": "boolean",      // 是否可行动
  "injured": "integer",   // 是否受伤 (持续伤害标记)
  "vote": "integer",      // 持有的票数
  "ts": "number",         // 上次搜索时间戳
  "deliver": "integer",   // 传音次数标记
  "rest": "integer"       // 静养模式标记
}
```

### 地点状态
```json
{
  "name": "string",
  "able": "boolean",      // 是否可用
  "exists": [],           // 在该地点的角色和道具
  "safe": "boolean"       // 是否为安全区
}
```

### 游戏规则配置（当前运行中的游戏）
```json
{
  "game_flow": {
    "day_duration": "integer",    // 白天时长(秒)
    "night_duration": "integer"   // 夜晚时长(秒)
  },
  "map": {
    "places": ["string"]          // 地点列表
  },
  "player": {
    "max_life": "integer",        // 最大生命值
    "max_strength": "integer",    // 最大体力值
    "daily_strength_recovery": "integer"  // 每日体力恢复值
  },
  "action": {
    "move_cost": "integer",       // 移动消耗体力
    "search_cost": "integer",     // 搜索消耗体力
    "search_cooldown": "integer"  // 搜索冷却时间(秒)
  },
  "rest_mode": {
    "life_recovery": "integer",   // 静养模式生命恢复值
    "max_moves": "integer"        // 静养模式最大移动次数
  },
  "teammate_behavior": "integer"  // 队友行为规则，位压缩存储
}
```

### 游戏规则模版配置（预设模版，可重用）
```json
{
  "template_name": "string",      // 模版名称
  "description": "string",        // 模版描述
  "game_flow": {
    "day_duration": "integer|null",    // 白天时长(秒)，null表示使用默认值
    "night_duration": "integer|null"   // 夜晚时长(秒)，null表示使用默认值
  },
  "map": {
    "places": ["string"]|null     // 地点列表，null表示使用默认地图
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
  },
  "teammate_behavior": "integer",      // 队友行为规则，位压缩存储
  "created_at": "ISO8601 datetime",    // 模版创建时间
  "updated_at": "ISO8601 datetime"     // 模版更新时间
}
```

### 游戏日志条目
```json
{
  "timestamp": "ISO8601 datetime",
  "level": "info|warn|error",
  "message": "string",
  "player": "string"  // 可选，相关玩家
}
```

### 游戏统计数据
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

### 投票数据
```json
{
  "voter": "string",      // 投票者
  "target": "string",     // 被投票者
  "timestamp": "ISO8601 datetime"
}
```

## 错误处理

所有 API 错误都会返回标准错误格式:
```json
{
  "error": {
    "code": "string",
    "message": "string"
  }
}
```