# 导演控制指令
```json
{
  "type": "director_action",
  "data": {
    "action": "max_life",
    "player_id": "玩家 ID",
    "max_life": 150
  }
}
```

## 控制参数说明:

**货币与背包上限：**

- `coins`：`player_id` 和 `coins`；金额必须是 `0..=4503599627370495.5` 内的半币数值。
- `sell_set_price`：`sell_rarity` 和 `sell_price`；价格必须是 `0.5..=9999` 内的 `0.5` 的倍数。
- `max_backpack`：`player_id` 和 `max_backpack_items`；容量通过 JSON 无符号整数校验，服务端使用 `usize` 全宽计算，再限制在规则基础容量与硬上限之间。前端应只发送 JavaScript 可精确表示的安全整数。

**本次新增的控制指令：** 以下字段均直接放在 `data` 中，与 `action` 同级。

| `action` | 必填字段 | 规则 |
| --- | --- | --- |
| `max_life` | `player_id`（字符串）、`max_life`（i32 整数） | 设置指定玩家生命上限；服务端将数值限制在规则基础上限与 `max_life_cap` 之间，必要时同步压低当前生命。 |
| `max_strength` | `player_id`（字符串）、`max_strength`（i32 整数） | 设置指定玩家体力上限；服务端将数值限制在规则基础上限与 `max_strength_cap` 之间，必要时同步压低当前体力。 |
| `set_teammate_behavior` | `teammate_behavior`（0 到 15 的整数） | 位 1 禁止队友伤害、位 2 搜索时过滤队友、位 4 允许查看队友状态、位 8 允许向队友转移物品；可相加组合，0 表示全部关闭。 |
| `shop_list_rarity` | `shop_item_kind`（`weapon` 或 `armor`）、`shop_rarity`（`common`、`rare`、`epic` 或 `legendary`）、`price`（至少 1 的 i32 整数） | `quantity` 可选，默认 1；必须不超过该类目道具库中的名称数量，同类目不能重复上架。 |
| `sell_remove_price` | `sell_rarity`（`common`、`rare`、`epic` 或 `legendary`） | 删除已配置的售出价格；未配置时返回提示。 |

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
