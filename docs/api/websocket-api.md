## WebSocket 接口

### 连接参数
连接时需要提供身份认证信息:
- `game_id`: 游戏ID
- `password`: 导演或玩家密码
- `user_type`: "player" 或 "director"

### 消息格式
所有 WebSocket 消息都使用 JSON 格式:
```json
{
  "type": "string",     // 消息类型
  "data": "object"      // 消息数据
}
```

### 客户端发送消息类型

#### 1. 玩家行动指令
```json
{
  "type": "player_action",
  "data": {
    "action": "born|move|search|pick|attack|equip|use|throw|deliver|send|rest",
    "params": {}  // 行动参数，根据具体行动类型而定
  }
}
```

##### 行动参数说明:

**出生 (born):**
```json
{
  "place": "string"  // 出生地点
}
```

**移动 (move):**
```json
{
  "target": "string"  // 目标地点
}
```

**搜索 (search):**
```json
{}  // 无需参数
```

**捡拾 (pick):**
```json
{}  // 无需参数
```

**攻击 (attack):**
```json
{
  "target": "string"  // 攻击目标角色名
}
```

**装备 (equip):**
```json
{
  "item": "string"  // 要装备的道具名
}
```

**使用道具 (use):**
```json
{
  "item": "string",          // 要使用的道具名
  "target_role": "string",   // 目标角色(可选)
  "target_item": "string",   // 目标道具(可选)
  "target_upgrade": "string" // 目标升级等级(可选)
}
```

**丢弃道具 (throw):**
```json
{
  "item": "string"  // 要丢弃的道具名
}
```

**传音 (deliver):**
```json
{
  "target": "string",  // 目标角色
  "content": "string"  // 消息内容
}
```

**对话导演 (send):**
```json
{
  "content": "string"  // 对话内容
}
```

**静养模式 (rest):**
```json
{
  "enable": "boolean"  // 是否启用静养模式
}
```

#### 2. 导演控制指令
```json
{
  "type": "director_action",
  "data": {
    "action": "start|end|save|jump|vote|destroy|drop|weather|life|strength|move_player|give|born_all|rope|unrope|broadcast|set_time|modify_map|reset_players|pause|resume|view_history|start_vote|end_vote",
    "params": {}  // 控制参数，根据具体控制类型而定
  }
}
```

##### 控制参数说明:

**开始行动 (start):**
```json
{}  // 无需参数
```

**结束行动 (end):**
```json
{}  // 无需参数
```

**存盘 (save):**
```json
{}  // 无需参数
```

**跳转到演员视角 (jump):**
```json
{
  "target": "string"  // 角色名称
}
```

**缴械 (vote):**
```json
{
  "target": "string"  // 角色名称
}
```

**缩圈 (destroy):**
```json
{
  "place": "string"  // 区域名称
}
```

**空投 (drop):**
```json
{
  "item": "string",  // 道具名称
  "place": "string"  // 区域名称
}
```

**调整天气 (weather):**
```json
{
  "value": "number"  // 搜索到人物能看到是谁的概率(0-1)
}
```

**加减生命 (life):**
```json
{
  "target": "string",  // 角色名称
  "value": "integer"   // 数值（正/负）
}
```

**加减体力 (strength):**
```json
{
  "target": "string",  // 角色名称
  "value": "integer"   // 数值（正/负）
}
```

**移动角色 (move_player):**
```json
{
  "target": "string",  // 角色名称
  "place": "string"    // 目的地名称
}
```

**增减道具 (give):**
```json
{
  "target": "string",  // 角色名称
  "item": "string"     // 道具名称
}
```

**随机出生 (born_all):**
```json
{}  // 无需参数
```

**捆绑（禁止行动）(rope):**
```json
{
  "target": "string"  // 角色名称
}
```

**松绑（取消禁令）(unrope):**
```json
{
  "target": "string"  // 角色名称
}
```

**广播消息 (broadcast):**
```json
{
  "message": "string"  // 广播内容
}
```

**设置游戏时间 (set_time):**
```json
{
  "day_duration": "integer",    // 白天时长(秒) - 可选
  "night_duration": "integer"   // 夜晚时长(秒) - 可选
}
```

**修改地图 (modify_map):**
```json
{
  "add_places": ["string"],     // 要添加的地点列表 - 可选
  "remove_places": ["string"]   // 要删除的地点列表 - 可选
}
```

**重置玩家状态 (reset_players):**
```json
{}  // 无需参数
```

**暂停游戏 (pause):**
```json
{}  // 无需参数
```

**恢复游戏 (resume):**
```json
{}  // 无需参数
```

**查看历史行动 (view_history):**
```json
{
  "player": "string",      // 玩家名称 - 可选
  "limit": "integer"       // 返回记录数限制 - 可选，默认10
}
```

**开始投票 (start_vote):**
```json
{
  "duration": "integer"    // 投票时长(秒) - 可选，使用默认白天时长
}
```

**结束投票 (end_vote):**
```json
{}  // 无需参数
```
