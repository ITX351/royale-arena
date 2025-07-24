# 导演控制指令
```json
{
  "type": "director_action",
  "data": {
    "action": "start|end|save|jump|vote|destroy|drop|weather|life|strength|move_player|give|born_all|rope|unrope|broadcast|set_time|modify_map|reset_players|pause|resume|view_history|start_vote|end_vote",
    "params": {}  // 控制参数，根据具体控制类型而定
  }
}
```

## 控制参数说明:

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
