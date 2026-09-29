# 游戏规则配置说明

## 概述

本文档详细说明了游戏规则配置JSON字段的结构和含义。该配置存储在`rule_templates`表的`rules_config`字段中。

## 规则配置结构

### 完整配置示例

```json
{
  "game_flow": {
    "day_duration": 300,
    "night_duration": 900
  },
  "map": {
    "places": [
      "码头", "工厂", "贫民窟", "旅馆", "教堂", "市政厅", "消防局", "池塘",
      "住宅区", "灯塔", "小巷", "学校", "隧道", "山道", "寺庙", "靶场",
      "医院", "森林", "海滩", "墓园", "井", "研究中心"
    ]
  },
  "player": {
    "max_life": 100,
    "max_strength": 100,
    "daily_strength_recovery": 40,
    "max_backpack_items": 6,
    "max_life_cap": 300,
    "max_strength_cap": 300,
    "max_backpack_items_cap": 12
  },
  "action": {
    "move_cost": 5,
    "search_cost": 5,
    "search_cooldown": 30
  },
  "rest_mode": {
    "life_recovery": 25,
    "max_moves": 1
  },
  "teammate_behavior": 0
}
```

### 字段详细说明

#### game_flow - 游戏流程配置
| 字段名 | 类型 | 说明 |
|--------|------|------|
| day_duration | integer | 白天时长(秒) |
| night_duration | integer | 夜晚时长(秒) |

#### map - 地图配置
| 字段名 | 类型 | 说明 |
|--------|------|------|
| places | string[] | 地点列表 |

#### player - 玩家配置
| 字段名 | 类型 | 说明 |
|--------|------|------|
| max_life | integer | 最大生命值 |
| max_strength | integer | 最大体力值 |
| daily_strength_recovery | integer | 每日体力恢复值 |
| max_backpack_items | integer | 背包容量基础值（玩家初始背包容量，可被永久增益道具提升） |
| max_life_cap | integer | 生命上限的硬上限（默认 300，旧配置缺省时取默认值） |
| max_strength_cap | integer | 体力上限的硬上限（默认 300，旧配置缺省时取默认值） |
| max_backpack_items_cap | integer | 背包容量的硬上限（默认 12，旧配置缺省时取默认值） |

#### action - 行动配置
| 字段名 | 类型 | 说明 |
|--------|------|------|
| move_cost | integer | 移动消耗体力 |
| search_cost | integer | 搜索消耗体力 |
| search_cooldown | integer | 搜索冷却时间(秒) |

#### rest_mode - 静养模式配置
| 字段名 | 类型 | 说明 |
|--------|------|------|
| life_recovery | integer | 静养模式生命恢复值 |
| max_moves | integer | 静养模式最大移动次数 |

#### teammate_behavior - 队友行为规则
| 值 | 说明 |
|----|------|
| 0 | 无限制 |
| 1 | 禁止队友伤害 |
| 2 | 禁止搜索到队友 |
| 4 | 允许观看队友状态 |
| 8 | 允许赠送队友物品 |

规则可以通过位运算组合，例如：
- 值为1：仅禁止队友伤害
- 值为5（1|4）：禁止队友伤害 + 允许观看队友状态
- 值为15（1|2|4|8）：启用所有队友行为规则

## 道具配置（items_config）

### 永久增益道具 (permanent_buffs)

`items_config.items.permanent_buffs` 数组定义永久增益道具，使用后永久提升对应属性的上限：

```json
{
  "items_config": {
    "items": {
      "permanent_buffs": [
        {"name": "[HP上限+20]养生丸", "properties": {"effect_type": "max_life", "effect_value": 20}},
        {"name": "[HP上限+50]壮骨丹", "properties": {"effect_type": "max_life", "effect_value": 50}},
        {"name": "[HP上限+100]金钟罩", "properties": {"effect_type": "max_life", "effect_value": 100}},
        {"name": "[MP上限+20]干粮", "properties": {"effect_type": "max_strength", "effect_value": 20}},
        {"name": "[MP上限+50]行军丹", "properties": {"effect_type": "max_strength", "effect_value": 50}},
        {"name": "[MP上限+100]龙力丸", "properties": {"effect_type": "max_strength", "effect_value": 100}},
        {"name": "[背包+2]腰包", "properties": {"effect_type": "max_backpack", "effect_value": 2}},
        {"name": "[背包+4]行囊", "properties": {"effect_type": "max_backpack", "effect_value": 4}},
        {"name": "[背包+6]百宝袋", "properties": {"effect_type": "max_backpack", "effect_value": 6}}
      ]
    }
  }
}
```

字段说明：
- `name`: 道具显示名称
- `internal_name`: 可选，道具的内部名称
- `rarity`: 可选，道具的稀有度
- `properties.effect_type`: `max_life`（生命上限）/ `max_strength`（体力上限）/ `max_backpack`（背包容量）
- `properties.effect_value`: 效果数值，正数提升上限，负数降低上限（削弱类道具，如 `{"effect_type": "max_life", "effect_value": -50}`）

规则：
- 提升时不恢复当前血量/体力；使用后道具消耗
- 溢出浪费：达到 `player` 配置的硬上限（`max_life_cap` / `max_strength_cap` / `max_backpack_items_cap`，默认 300/300/12）后再使用，超出部分无效，道具照常消耗
- 生效上限 = max(硬上限, 规则基础值)，正数道具永远不会让上限超过生效上限
- 负数道具降低上限，下限为规则基础值（`max_life` / `max_strength` / `max_backpack_items` 的初始值），不会低于开局状态；降低后当前血量/体力若超过新上限，压到新上限
- 旧配置无 `permanent_buffs` 数组时解析为空列表，保持兼容

## 扩展性说明

当需要添加新的规则字段时，可以直接在相应的对象中添加新字段，无需修改数据库表结构。例如：

```json
{
  "game_flow": {
    "day_duration": 300,
    "night_duration": 900
  },
  "new_feature": {
    "special_rule": true,
    "bonus_value": 10
  }
  // ... 其他规则
}
```