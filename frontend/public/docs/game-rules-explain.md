# Royale Arena 游戏规则配置说明

## 配置文件结构

游戏规则配置文件采用分层的JSON结构：

```json
{
  "map": {
    "places": [],
    "safe_places": []
  },
  "player": {
    "max_life": 100,
    "max_strength": 100,
    "daily_life_recovery": 0,
    "daily_strength_recovery": 40,
    "search_cooldown": 30,
    "max_backpack_items": 6,
    "unarmed_damage": 5
  },
  "action_costs": {
    "move": 5,
    "search": 5,
    "pick": 0,
    "attack": 0,
    "equip": 0,
    "use": 0,
    "throw": 0,
    "deliver": 10
  },
  "rest_mode": {
    "life_recovery": 25,
    "strength_recovery": 1000,
    "max_moves": 1
  },
  "death_item_disposition": "killer_takes_loot",
  "items_config": {
    "rarity_levels": [],
    "items": {
      "weapons": [],
      "armors": [],
      "utilities": [],
      "consumables": [],
      "upgraders": []
    },
    "upgrade_recipes": {}
  },
  "teammate_behavior": 0,
  "display_names": {
    "player_max_life": "生命值",
    "player_max_strength": "体力值",
    "player_daily_life_recovery": "每日生命恢复",
    "player_daily_strength_recovery": "每日体力恢复",
    "player_search_cooldown": "搜索冷却时间",
    "action_move": "移动",
    "action_search": "搜索",
    "action_pick": "拾取",
    "action_attack": "攻击",
    "action_equip": "装备",
    "action_use": "使用",
    "action_throw": "丢弃",
    "action_deliver": "传音",
    "rest_life_recovery": "生命恢复",
    "rest_max_moves": "最大移动次数"
  }
}
```

## 1. 基础设置

### 地图配置 (map)

定义游戏中的地点和安全区域。

```json
{
  "map": {
    "places": [
      "码头", "工厂", "贫民窟", "旅馆", "教堂", "市政厅", "消防局", "池塘",
      "住宅区", "灯塔", "小巷", "学校", "隧道", "山道", "寺庙", "靶场",
      "医院", "森林", "海滩", "墓园", "井", "研究中心"
    ],
    "safe_places": ["研究中心"]
  }
}
```

字段说明：
- `places`: 游戏中的所有地点列表
- `safe_places`: 安全区域列表

### 玩家属性 (player)

定义玩家的基础属性和装备限制。

```json
{
  "player": {
    "max_life": 100,
    "max_strength": 100,
    "daily_life_recovery": 0,
    "daily_strength_recovery": 40,
    "search_cooldown": 30,
    "max_backpack_items": 6,
    "unarmed_damage": 5
  }
}
```

字段说明：
- `max_life`: 玩家的最大生命值
- `max_strength`: 玩家的最大体力值
- `daily_life_recovery`: 每天白天恢复的生命值
- `daily_strength_recovery`: 每天白天恢复的体力值
- `search_cooldown`: 搜索行动的冷却时间（秒）
- `max_backpack_items`: 背包中最多允许存放的物品数量（包括已装备的武器和防具）
- `unarmed_damage`: 无武器攻击时的挥拳伤害值

## 2. 行动规则

### 行动消耗 (action_costs)

定义各种行动的体力消耗。

```json
{
  "action_costs": {
    "move": 5,
    "search": 5,
    "pick": 0,
    "attack": 0,
    "equip": 0,
    "use": 0,
    "throw": 0,
    "deliver": 10
  }
}
```

字段说明：
- `move`: 移动到其他地点消耗的体力
- `search`: 搜索当前地点消耗的体力
- `pick`: 拾取物品消耗的体力
- `attack`: 攻击其他玩家消耗的体力
- `equip`: 装备物品消耗的体力
- `use`: 使用物品消耗的体力
- `throw`: 丢弃物品消耗的体力
- `deliver`: 发送消息消耗的体力

### 静养模式 (rest_mode)

定义玩家进入静养模式时的效果。

```json
{
  "rest_mode": {
    "life_recovery": 25,
    "strength_recovery": 1000,
    "max_moves": 1
  }
}
```

字段说明：
- `life_recovery`: 静养模式下恢复的生命值
- `strength_recovery`: 静养模式下恢复的体力值
- `max_moves`: 静养模式下每晚最多可移动的次数

## 3. 物品系统 (items_config)

### 稀有度等级 (rarity_levels)

定义物品的稀有度等级及其空投规则。

```json
{
  "items_config": {
    "rarity_levels": [
      {"internal_name": "common", "display_name": "普通", "prefix": "[绿]", "is_airdropped": true},
      {"internal_name": "rare", "display_name": "稀有", "prefix": "[蓝]", "is_airdropped": true},
      {"internal_name": "epic", "display_name": "史诗", "prefix": "[紫]", "is_airdropped": false},
      {"internal_name": "legendary", "display_name": "传说", "prefix": "[橙]", "is_airdropped": false}
    ]
  }
}
```

字段说明：
- `internal_name`: 稀有度的内部标识符
- `display_name`: 显示给玩家的名称
- `prefix`: 物品名称前的标识前缀
- `is_airdropped`: 该稀有度的物品是否会被随机空投（蓝绿会被空投、紫橙不会被自动空投）

### 武器装备 (weapons)

定义游戏中的武器装备。

```json
{
  "items_config": {
    "items": {
      "weapons": [
        {
          "internal_name": "common_weapon",
          "display_names": ["[绿]佩剑", "[绿]战斧", "[绿]长矛", "[绿]皮鞭", "[绿]回力镖", "[绿]IM-10", "[绿]复合弓", "[绿]铁爪"],
          "rarity": "common",
          "properties": {
            "damage": 10,
            "votes": 1
          }
        },
        {
          "internal_name": "legendary_weapon",
          "display_names": ["[橙]自然之力.晓", "[橙]自然之力.午", "[橙]自然之力.夜", "[橙]自然之力.日", "[橙]自然之力.月", "[橙]自然之力.星", "[橙]自然之力.水", "[橙]自然之力.火", "[橙]自然之力.风"],
          "rarity": "legendary",
          "properties": {
            "damage": 40,
            "uses": 5,
            "votes": 0,
            "aoe_damage": 40,
            "bleed_damage": 10
          }
        }
      ]
    }
  }
}
```

字段说明：
- `internal_name`: 武器的内部标识符
- `display_names`: 武器的显示名称列表
- `rarity`: 武器的稀有度等级
- `properties`: 武器的属性
  - `damage`: 攻击伤害值
  - `uses`: 使用次数（无限使用的武器不包含此字段）
  - `votes`: 攻击时获得的票数加成
  - `aoe_damage`: 范围伤害值
  - `bleed_damage`: 持续伤害值

橙色终极武器说明：在攻击目标本体的同时，对所在区域的其他角色也造成伤害（`aoe_damage`）。这些角色之后每天清晨会受到持续伤害（`bleed_damage`），可通过使用药品抵消。武器限用`uses`次。

### 护甲装备 (armors)

定义游戏中的护甲装备。

```json
{
  "items_config": {
    "items": {
      "armors": [
        {
          "internal_name": "common_armor",
          "display_names": ["[绿]皮甲", "[绿]布衣", "[绿]轻甲"],
          "rarity": "common",
          "properties": {
            "defense": 5,
            "votes": 2
          }
        }
      ]
    }
  }
}
```

字段说明：
- `internal_name`: 护甲的内部标识符
- `display_names`: 护甲的显示名称列表
- `rarity`: 护甲的稀有度等级
- `properties`: 护甲的属性
  - `defense`: 防御力值
  - `votes`: 被攻击时获得的票数加成
  - `uses`: 使用次数（无限使用的道具不包含此字段）

### 功能道具 (utilities)

定义游戏中的其他类型道具。

```json
{
  "items_config": {
    "items": {
      "utilities": [
        {
          "name": "[控]电击棒",
          "properties": {
            "category": "utility_control",
            "votes": 3,
            "uses_night": 1
          }
        },
        {
          "name": "[GPS]心跳探测仪",
          "properties": {
            "category": "utility_locator",
            "votes": 3,
            "targets": 1,
            "uses_night": 1
          }
        },
        {
          "name": "[侦]手持式雷达",
          "properties": {
            "category": "utility_revealer",
            "votes": 3,
            "targets": 2,
            "uses_night": 1
          }
        },
        {
          "name": "[神]生命启示",
          "properties": {
            "category": "utility_seer",
            "votes": 3,
            "targets": 2
          }
        },
        {
          "name": "[炸]遥控地雷",
          "properties": {
            "category": "utility_trap",
            "damage": 30,
            "uses": 1,
            "votes": 0
          }
        }
      ]
    }
  }
}
```

字段说明：
- `name`: 道具的显示名称
- `internal_name`: 可选，便于编写流程脚本时引用的内部名称
- `rarity`: 可选，标识该道具的稀有度（部分工具可能也跟随空投规则）
- `properties`: 道具的属性
  - `category`: 道具的分类
    - `utility_control`: 控制类道具（如电击棒）
    - `utility_locator`: 定位工具类道具（如心跳探测仪，用于探测目标位置）
    - `utility_revealer`: 揭示工具类道具（如手持式雷达，用于揭示目标信息）
    - `utility_seer`: 预言工具类道具（如生命启示，用于预知位置信息）
    - `utility_trap`: 陷阱类道具（如遥控地雷）
  - `uses`: 使用次数（对于有限使用次数的道具，如遥控地雷；无限使用的道具不包含此字段）
  - `votes`: 使用时获得的票数加成
  - `targets`: 可作用的目标数量（仅适用于定位、揭示和预言类道具）
  - `uses_night`: 每晚可使用次数（未配置时默认无限制）
  - `damage`: 造成的伤害值（仅适用于陷阱类道具）

各类道具详细功能说明：
1. `[控]电击棒`, `utility_control`
   - 每晚限用一次，立即令同区域的其他角色进入“被捆绑”状态。
2. `[GPS]心跳探测仪`, `utility_locator`
   - 每晚限用一次，可查询1件道具当前所在区域；若该道具被角色装备，则反馈该角色所在区域。
3. `[侦]手持式雷达`, `utility_revealer`
   - 每晚限用一次，可查询2名目标角色持有的全部道具名称。
4. `[炸]遥控地雷`, `utility_trap`
   - 单次使用后消失，使当前所在区域的所有其他角色生命值减少30点（造成击杀时无伤害来源）。

> 提示：`[神]生命启示`目前不提供额外效果实现，仅保留配置项以兼容旧数据。

### 消耗品 (consumables)

定义游戏中的消耗品道具。

```json
{
  "items_config": {
    "items": {
      "consumables": [
        {
          "name": "[HP30]绷带",
          "properties": {
            "effect_type": "heal",
            "effect_value": 30,
            "cure_bleed": 1
          }
        },
        {
          "name": "[HP100]红花丹",
          "properties": {
            "effect_type": "heal",
            "effect_value": 100,
            "cure_bleed": 2
          }
        },
        {
          "name": "[MP20]矿泉水",
          "properties": {
            "effect_type": "strength",
            "effect_value": 20
          }
        }
      ]
    }
  }
}
```

字段说明：
- `name`: 消耗品的显示名称
- `internal_name`: 可选，消耗品的内部名称
- `rarity`: 可选，消耗品的稀有度
- `properties`: 消耗品的效果配置
  - `effect_type`: 效果类型（heal=治疗, strength=恢复体力）
  - `effect_value`: 效果数值
  - `cure_bleed`: 是否能解除持续伤害，0或缺省为无法解除，1为成功解除流血时不加血，2为完全解除流血并增加生命值

消耗品详细功能说明：
1. 治疗类消耗品
   - 绷带：使用时，若角色身上有持续伤害，会优先抵消持续伤害，不再增加生命值
   - 红花丹：使用时，若角色身上有持续伤害，会同时抵消持续伤害，生命值也会增加

2. 体力恢复类消耗品
   - 矿泉水：使用后恢复指定体力值

### 合成系统 (upgraders & upgrade_recipes)

定义物品的合成配方。

```json
{
  "items_config": {
    "items": {
      "upgraders": [
        {
          "internal_name": "natural_upgrader",
          "display_names": ["[合]自然升级器"],
          "rarity": "legendary"
        },
        {
          "internal_name": "artificial_upgrader",
          "display_names": ["[合]人造升级器"],
          "rarity": "rare"
        }
      ]
    },
    "upgrade_recipes": {
      "natural_upgrader": [
        {
          "result": "rare_weapon",
          "ingredients": ["common_weapon"]
        }
      ]
    }
  }
}
```

字段说明：
- `upgraders`: 升级道具的定义（每个升级器只能使用一次）
  - `internal_name`: 升级道具的内部标识符
  - `display_names`: 升级道具的显示名称
  - `rarity`: 升级道具的稀有度
- `upgrade_recipes`: 合成配方（键为升级器的internal_name）
  - `result`: 合成结果（目标物品的internal_name）
  - `ingredients`: 所需材料（原料物品的internal_name列表）

升级器详细功能说明：
1. 自然升级器：可进行任何升级，包括将紫色武器升级为橙色终极武器
2. 人造升级器：最多只能升级到紫色武器，无法合成橙色终极武器

## 4. 显示名称配置 (display_names)

定义前端界面中显示的中文标签名称。

```json
{
  "display_names": {
    "player_max_life": "生命值",
    "player_max_strength": "体力值",
    "player_daily_life_recovery": "每日生命恢复",
    "player_daily_strength_recovery": "每日体力恢复",
    "player_search_cooldown": "搜索冷却时间",
    "action_move": "移动",
    "action_search": "搜索",
    "action_pick": "拾取",
    "action_attack": "攻击",
    "action_equip": "装备",
    "action_use": "使用",
    "action_throw": "丢弃",
    "action_deliver": "传音",
    "rest_life_recovery": "生命恢复",
    "rest_max_moves": "最大移动次数"
  }
}
```

字段说明：
- `player_max_life`: 生命值显示名称
- `player_max_strength`: 体力值显示名称
- `player_daily_life_recovery`: 每日生命恢复显示名称
- `player_daily_strength_recovery`: 每日体力恢复显示名称
- `player_search_cooldown`: 搜索冷却时间显示名称
- `action_move`: 移动显示名称
- `action_search`: 搜索显示名称
- `action_pick`: 拾取显示名称
- `action_attack`: 攻击显示名称
- `action_equip`: 装备显示名称
- `action_use`: 使用显示名称
- `action_throw`: 丢弃显示名称
- `action_deliver`: 传音显示名称
- `action_unarmed_damage`: 挥拳伤害显示名称
- `rest_life_recovery`: 生命恢复显示名称
- `rest_max_moves`: 最大移动次数显示名称

## 5. 高级设置

### 队友行为 (teammate_behavior)

定义队友之间的行为规则（使用位运算值）。

```json
{
  "teammate_behavior": 0
}
```

字段说明：
- `teammate_behavior`: 队友行为规则，通过数字组合实现不同效果：
  - `0`: 无特殊规则（默认）
  - `1`: 禁止队友伤害
  - `2`: 禁止搜索到队友
  - `4`: 允许查看队友状态
  - `8`: 允许赠送物品给队友
  - 可以通过相加组合规则，例如：`5`（1+4）表示禁止伤害且允许查看状态

### 死亡后物品去向 (death_item_disposition)

定义玩家死亡后其物品的处理方式。

```json
{
  "death_item_disposition": "killer_takes_loot"
}
```

字段说明：
- `death_item_disposition`: 玩家死亡后物品的处理方式，可选值：
  - `"killer_takes_loot"`: 如果有击杀者，则物品由击杀者获得；如果没有击杀者（如因环境因素死亡），则物品掉落在原地
  - `"drop_to_ground"`: 玩家死亡后，其所有物品均掉落在死亡地点
  - `"vanish_completely"`: 玩家死亡后，其所有物品直接消失，不留下任何物品