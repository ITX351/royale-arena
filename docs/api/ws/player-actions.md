
# 玩家行动指令
```json
{
  "type": "player_action",
  "data": {
    "action": "born|move|search|pick|attack|equip|use|throw|deliver|send|rest",
    "params": {}  // 行动参数，根据具体行动类型而定
  }
}
```

## 行动参数说明:

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

使用道具补充说明：
- 对 `permanent_buff` 类型道具（永久增益），返回的 data 含对应属性及其变化量：
  - 生命上限类：`max_life` + `max_life_delta`
  - 体力上限类：`max_strength` + `max_strength_delta`
  - 背包容量类：`max_backpack_items` + `max_backpack_items_delta`
  - 同时返回 `strength` / `strength_delta`（使用本身的体力消耗，默认 0）
- 道具使用后消耗；达到硬上限后再使用，超出部分无效，道具照常消耗
- 未知 `effect_type` 返回 Info 错误，道具退回背包

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
