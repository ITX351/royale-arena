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
    "daily_strength_recovery": 40
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