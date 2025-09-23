# 规则模版API

<cite>
**本文档引用的文件**  
- [models.rs](file://backend/src/rule_template/models.rs#L1-L243)
- [handlers.rs](file://backend/src/rule_template/handlers.rs#L1-L66)
- [service.rs](file://backend/src/rule_template/service.rs#L1-L229)
- [errors.rs](file://backend/src/rule_template/errors.rs#L1-L75)
- [routes.rs](file://backend/src/routes.rs#L1-L71)
- [middleware.rs](file://backend/src/auth/middleware.rs#L1-L62)
- [.qoder/quests/game-rule-refactor-and-frontend-display.md](file://.qoder/quests/game-rule-refactor-and-frontend-display.md) - *新增：规则JSON设计文档*
- [return.json](file://backend/json_sample/return.json) - *新增：规则配置示例文件*
</cite>

## 更新摘要
**变更内容**
- 根据新的游戏规则JSON结构更新了`rules_config`字段的详细说明
- 新增了规则配置的完整示例，展示复杂的物品系统和稀有度级别
- 更新了使用示例中的请求体，以反映更丰富的规则配置
- 增加了对前端解析模块的说明，解释如何处理复杂的规则JSON

**Section sources**
- [.qoder/quests/game-rule-refactor-and-frontend-display.md](file://.qoder/quests/game-rule-refactor-and-frontend-display.md) - *新增：规则JSON设计文档*
- [return.json](file://backend/json_sample/return.json) - *新增：规则配置示例文件*

## 目录
1. [简介](#简介)
2. [数据模型](#数据模型)
3. [核心API端点](#核心api端点)
4. [认证要求](#认证要求)
5. [错误码](#错误码)
6. [使用示例](#使用示例)

## 简介

规则模版API是Royale Arena后端系统中的核心管理接口，用于创建、更新和查询游戏规则模版。该API支持管理员用户对游戏规则进行灵活配置，同时允许普通用户查询当前可用的规则模版。API设计遵循RESTful原则，提供统一的请求响应格式和详细的错误处理机制。

**Section sources**
- [routes.rs](file://backend/src/routes.rs#L1-L71)

## 数据模型

### 规则模版实体 (RuleTemplate)

规则模版是系统中的核心数据结构，用于存储游戏规则的配置信息。

```
classDiagram
class RuleTemplate {
+string id
+string template_name
+string? description
+bool is_active
+json rules_config
+datetime created_at
+datetime updated_at
}
class RuleTemplateResponse {
+string id
+string template_name
+string? description
+bool is_active
+json rules_config
+datetime created_at
+datetime updated_at
}
class CreateRuleTemplateRequest {
+string template_name
+string? description
+bool? is_active
+json rules_config
}
class UpdateRuleTemplateRequest {
+string? template_name
+string? description
+bool? is_active
+json? rules_config
}
class GetTemplatesQuery {
+string? id
+bool? is_active
+string? search
}
RuleTemplateResponse <|-- RuleTemplate : "转换"
CreateRuleTemplateRequest --> RuleTemplate : "创建"
UpdateRuleTemplateRequest --> RuleTemplate : "更新"
GetTemplatesQuery --> RuleTemplate : "查询"
```

**Diagram sources**
- [models.rs](file://backend/src/rule_template/models.rs#L1-L243)

**Section sources**
- [models.rs](file://backend/src/rule_template/models.rs#L1-L243)

#### 字段说明

- **id**: 模版唯一标识符，使用UUID生成
- **template_name**: 模版名称，长度限制100字符，必须唯一
- **description**: 模版描述，可选字段
- **is_active**: 模版是否激活状态
- **rules_config**: 规则配置，存储为JSON对象，包含具体的游戏规则设置，包括地图、玩家属性、行动消耗、物品系统等复杂结构
- **created_at**: 创建时间戳
- **updated_at**: 更新时间戳

#### 请求/响应模型

- **CreateRuleTemplateRequest**: 创建模版时的请求体，所有字段均为必填（is_active可选，默认为true）
- **UpdateRuleTemplateRequest**: 更新模版时的请求体，所有字段均为可选，但至少需要提供一个字段
- **RuleTemplateResponse**: 统一的响应格式，包含完整的模版信息
- **GetTemplatesQuery**: 查询参数，支持按ID、激活状态和名称搜索进行过滤

## 核心API端点

### 创建规则模版

**端点**: `POST /api/admin/rule-templates`  
**权限**: 管理员  
**功能**: 创建新的规则模版

```
sequenceDiagram
participant Client as "客户端"
participant Handler as "创建处理器"
participant Service as "业务服务"
participant DB as "数据库"
Client->>Handler : POST /api/admin/rule-templates
Handler->>Handler : 解析JSON请求体
Handler->>Service : 调用create_template()
Service->>Service : 验证请求数据
Service->>Service : 检查名称唯一性
Service->>DB : 插入新记录
DB-->>Service : 返回结果
Service->>Service : 查询新创建的模版
Service-->>Handler : 返回RuleTemplateResponse
Handler-->>Client : 201 Created + 数据
```

**Diagram sources**
- [handlers.rs](file://backend/src/rule_template/handlers.rs#L1-L66)
- [service.rs](file://backend/src/rule_template/service.rs#L1-L229)

**Section sources**
- [handlers.rs](file://backend/src/rule_template/handlers.rs#L1-L66)
- [service.rs](file://backend/src/rule_template/service.rs#L1-L229)

### 更新规则模版

**端点**: `PUT /api/admin/rule-templates/{id}`  
**权限**: 管理员  
**功能**: 更新现有规则模版

```
sequenceDiagram
participant Client as "客户端"
participant Handler as "更新处理器"
participant Service as "业务服务"
participant DB as "数据库"
Client->>Handler : PUT /api/admin/rule-templates/{id}
Handler->>Handler : 解析路径参数和请求体
Handler->>Service : 调用update_template()
Service->>Service : 验证请求数据
Service->>Service : 检查模版是否存在
Service->>Service : 检查名称唯一性如更新名称
Service->>DB : 执行动态更新
DB-->>Service : 返回结果
Service->>Service : 查询更新后的模版
Service-->>Handler : 返回RuleTemplateResponse
Handler-->>Client : 200 OK + 数据
```

**Diagram sources**
- [handlers.rs](file://backend/src/rule_template/handlers.rs#L1-L66)
- [service.rs](file://backend/src/rule_template/service.rs#L1-L229)

**Section sources**
- [handlers.rs](file://backend/src/rule_template/handlers.rs#L1-L66)
- [service.rs](file://backend/src/rule_template/service.rs#L1-L229)

### 查询规则模版

**端点**: `GET /api/rule-templates`  
**权限**: 公开  
**功能**: 查询规则模版列表，支持多种过滤条件

```
sequenceDiagram
participant Client as "客户端"
participant Handler as "查询处理器"
participant Service as "业务服务"
participant DB as "数据库"
Client->>Handler : GET /api/rule-templates
Handler->>Service : 调用get_templates()
Service->>Service : 检查是否有ID参数
alt 有ID参数
Service->>DB : 按ID查询单个模版
else 无ID参数
Service->>Service : 构建条件查询
Service->>DB : 执行条件查询
end
DB-->>Service : 返回结果集
Service->>Service : 转换为Response格式
Service-->>Handler : 返回模版列表
Handler-->>Client : 200 OK + 数据列表
```

**Diagram sources**
- [handlers.rs](file://backend/src/rule_template/handlers.rs#L1-L66)
- [service.rs](file://backend/src/rule_template/service.rs#L1-L229)

**Section sources**
- [handlers.rs](file://backend/src/rule_template/handlers.rs#L1-L66)
- [service.rs](file://backend/src/rule_template/service.rs#L1-L229)

## 认证要求

规则模版API的认证机制基于JWT（JSON Web Token），不同端点有不同的权限要求：

```
flowchart TD
Start([请求进入]) --> ExtractToken["提取Authorization头"]
ExtractToken --> ValidateToken{"Token有效?"}
ValidateToken --> |否| Return401["返回401 Unauthorized"]
ValidateToken --> |是| VerifyClaims["验证Token声明"]
VerifyClaims --> CheckAdmin{"是否管理员端点?"}
CheckAdmin --> |否| AllowPublic["允许访问公开端点"]
CheckAdmin --> |是| CheckPermissions["检查管理员权限"]
CheckPermissions --> IsAdmin{"是否管理员?"}
IsAdmin --> |否| Return403["返回403 Forbidden"]
IsAdmin --> |是| AllowAccess["允许访问管理员端点"]
AllowPublic --> ProcessRequest["处理请求"]
AllowAccess --> ProcessRequest
ProcessRequest --> ReturnResponse["返回响应"]
```

**Diagram sources**
- [routes.rs](file://backend/src/routes.rs#L1-L71)
- [middleware.rs](file://backend/src/auth/middleware.rs#L1-L62)

**Section sources**
- [routes.rs](file://backend/src/routes.rs#L1-L71)
- [middleware.rs](file://backend/src/auth/middleware.rs#L1-L62)

### 权限说明

- **创建和更新端点** (`/api/admin/rule-templates/*`): 需要管理员身份认证，通过JWT Token验证
- **查询端点** (`/api/rule-templates`): 公开访问，无需认证，所有用户均可查询规则模版

### 认证流程

1. 客户端在请求头中包含 `Authorization: Bearer <token>` 
2. 系统通过 `jwt_auth_middleware` 中间件验证Token的有效性
3. 验证通过后，用户信息（JWT Claims）被注入请求上下文中
4. 对于管理员端点，系统检查用户是否具有管理员权限

## 错误码

规则模版API提供详细的错误处理机制，所有错误响应都遵循统一的格式：

```json
{
  "success": false,
  "error": {
    "message": "错误信息",
    "details": "详细错误描述"
  }
}
```

### 错误类型

```
erDiagram
ERROR_TYPES {
string error_code PK
string message
int http_status
string description
}
ERROR_TYPES ||--o{ NAME_ALREADY_EXISTS : "模版名称已存在"
ERROR_TYPES ||--o{ TEMPLATE_NOT_FOUND : "模版不存在"
ERROR_TYPES ||--o{ VALIDATION_ERROR : "参数验证失败"
ERROR_TYPES ||--o{ DATABASE_ERROR : "服务器内部错误"
NAME_ALREADY_EXISTS {
string error_code "NAME_ALREADY_EXISTS"
string message "模版名称已存在"
int http_status 409
string description "尝试创建或更新时使用了已存在的模版名称"
}
TEMPLATE_NOT_FOUND {
string error_code "TEMPLATE_NOT_FOUND"
string message "模版不存在"
int http_status 404
string description "指定ID的规则模版不存在"
}
VALIDATION_ERROR {
string error_code "VALIDATION_ERROR"
string message "参数验证失败"
int http_status 400
string description "请求参数不符合验证规则"
}
DATABASE_ERROR {
string error_code "DATABASE_ERROR"
string message "服务器内部错误"
int http_status 500
string description "数据库操作失败"
}
```

**Diagram sources**
- [errors.rs](file://backend/src/rule_template/errors.rs#L1-L75)

**Section sources**
- [errors.rs](file://backend/src/rule_template/errors.rs#L1-L75)

### HTTP状态码映射

- **200 OK**: 请求成功，返回查询结果
- **201 Created**: 创建成功，返回新创建的资源
- **400 Bad Request**: 请求参数验证失败
- **401 Unauthorized**: 认证失败，Token无效或缺失
- **403 Forbidden**: 权限不足，非管理员尝试访问管理端点
- **404 Not Found**: 请求的资源不存在
- **409 Conflict**: 资源冲突，如模版名称已存在
- **500 Internal Server Error**: 服务器内部错误

## 使用示例

### 创建规则模版

```bash
curl -X POST https://api.royale-arena.com/api/admin/rule-templates \
  -H "Authorization: Bearer your-jwt-token" \
  -H "Content-Type: application/json" \
  -d '{
    "template_name": "标准竞技场规则",
    "description": "适用于标准5v5对战的规则配置",
    "is_active": true,
    "rules_config": {
      "map": {
        "places": ["码头", "工厂", "贫民窟", "旅馆", "教堂", "市政厅", "消防局", "池塘", "住宅区", "灯塔", "小巷", "学校", "隧道", "山道", "寺庙", "靶场", "医院", "森林", "海滩", "墓园", "井", "研究中心"],
        "safe_places": ["研究中心"]
      },
      "player": {
        "max_life": 100,
        "max_strength": 100,
        "daily_strength_recovery": 40,
        "search_cooldown": 30
      },
      "action_costs": {
        "move": 5,
        "search": 5,
        "pick": 0,
        "attack": 0,
        "equip": 0,
        "use": 0,
        "throw": 0,
        "deliver": 105
      },
      "rest_mode": {
        "life_recovery": 25,
        "max_moves": 1
      },
      "teammate_behavior": 0,
      "items": {
        "rarity_levels": [
          {"name": "common", "display_name": "普通", "prefix": "[绿]"},
          {"name": "rare", "display_name": "稀有", "prefix": "[蓝]"},
          {"name": "epic", "display_name": "史诗", "prefix": "[紫]"},
          {"name": "legendary", "display_name": "传说", "prefix": "[橙]"}
        ],
        "categories": [
          {"name": "weapon_melee", "display_name": "近战武器"},
          {"name": "weapon_ranged", "display_name": "远程武器"},
          {"name": "weapon_aoe", "display_name": "范围武器"},
          {"name": "consumable_heal", "display_name": "治疗道具"},
          {"name": "consumable_buff", "display_name": "增益道具"},
          {"name": "consumable_debuff", "display_name": "减益道具"},
          {"name": "equipment_armor", "display_name": "护甲装备"},
          {"name": "utility_vision", "display_name": "侦察道具"},
          {"name": "utility_control", "display_name": "控制道具"},
          {"name": "upgrade", "display_name": "升级道具"}
        ],
        "weapons": [
          {
            "internal_name": "common_weapon",
            "display_names": ["[绿]佩剑", "[绿]战斧", "[绿]长矛", "[绿]皮鞭", "[绿]回力镖", "[绿]IM-10", "[绿]复合弓", "[绿]铁爪"],
            "rarity": "common",
            "properties": {
              "damage": 10,
              "uses": 10000,
              "votes": 1
            }
          },
          {
            "internal_name": "rare_weapon",
            "display_names": ["[蓝]大太刀", "[蓝]死神镰刀", "[蓝]斩马刀", "[蓝]三叉戟", "[蓝]带电短刀", "[蓝]西洋剑", "[蓝]双节棍", "[蓝]荆棘之鞭", "[蓝]白羽扇", "[蓝]燃烧弹", "[蓝]复古扑克", "[蓝]强力回力镖", "[蓝]轻机枪", "[蓝]斯太尔AUG", "[蓝]AK-47", "[蓝]十字弩", "[蓝]诸葛连弩", "[蓝]火矢弓", "[蓝]铁砂掌", "[蓝]羽翼指虎", "[蓝]恶魔之爪"],
            "rarity": "rare",
            "properties": {
              "damage": 20,
              "uses": 10000,
              "votes": 0
            }
          },
          {
            "internal_name": "epic_weapon",
            "display_names": ["[紫]青龙偃月刀", "[紫]盘古斧", "[紫]宇宙双叉戟", "[紫]芭蕉扇", "[紫]风魔手里剑", "[紫]蔚蓝匕首", "[紫]北极星", "[紫]魔弹射手", "[紫]丘比特之弓", "[紫]费尔努特", "[紫]血翼指虎", "[紫]裁决之光"],
            "rarity": "epic",
            "properties": {
              "damage": 30,
              "uses": 10000,
              "votes": 0
            }
          },
          {
            "internal_name": "legendary_weapon",
            "display_names": ["[橙]自然之力.晓", "[橙]自然之力.午", "[橙]自然之力.夜", "[橙]自然之力.日", "[橙]自然之力.月", "[橙]自然之力.星", "[橙]自然之力.水", "[橙]自然之力.火", "[橙]自然之力.风"],
            "rarity": "legendary",
            "properties": {
              "damage": 40,
              "damage_lasted": 10,
              "uses": 5,
              "votes": 0
            }
          }
        ],
        "armors": [
          {
            "internal_name": "common_armor",
            "display_names": ["[绿]皮甲", "[绿]布衣", "[绿]轻甲"],
            "rarity": "common",
            "properties": {
              "defense": 5,
              "uses": 10000,
              "votes": 2
            }
          },
          {
            "internal_name": "rare_armor",
            "display_names": ["[蓝]锁子甲", "[蓝]鳞甲", "[蓝]链甲"],
            "rarity": "rare",
            "properties": {
              "defense": 10,
              "uses": 10000,
              "votes": 2
            }
          },
          {
            "internal_name": "epic_armor",
            "display_names": ["[紫]板甲", "[紫]重甲", "[紫]龙鳞甲"],
            "rarity": "epic",
            "properties": {
              "defense": 15,
              "uses": 10000,
              "votes": 2
            }
          },
          {
            "internal_name": "legendary_armor",
            "display_names": ["[橙]神佑之铠", "[橙]不朽战甲", "[橙]星辰护甲"],
            "rarity": "legendary",
            "properties": {
              "defense": 20,
              "uses": 5,
              "votes": 2
            }
          }
        ],
        "other_items": [
          {
            "name": "[GPS]心跳探测仪1",
            "category": "utility_vision",
            "properties": {
              "uses": 1,
              "votes": 0
            }
          },
          {
            "name": "[侦]手持式雷达1",
            "category": "utility_vision",
            "properties": {
              "uses": 2,
              "votes": 0
            }
          }
        ],
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
        ],
        "upgrade_recipes": {
          "natural_upgrader": [
            {
              "result": "rare_weapon",
              "ingredients": ["common_weapon"]
            },
            {
              "result": "epic_weapon",
              "ingredients": ["rare_weapon"]
            },
            {
              "result": "legendary_weapon",
              "ingredients": ["epic_weapon"]
            }
          ],
          "artificial_upgrader": [
            {
              "result": "rare_weapon",
              "ingredients": ["common_weapon"]
            },
            {
              "result": "epic_weapon",
              "ingredients": ["rare_weapon"]
            }
          ]
        },
        "consumable_effects": {
          "heal": [
            {
              "name": "[HP100]红花丹",
              "effect_type": "heal",
              "effect_value": 100,
              "cure_injury": true
            }
          ],
          "strength": [
            {
              "name": "[MP100]威士忌",
              "effect_type": "strength",
              "effect_value": 100
            }
          ]
        }
      }
    }
  }'
```

### 更新规则模版

```bash
curl -X PUT https://api.royale-arena.com/api/admin/rule-templates/abc123-def456 \
  -H "Authorization: Bearer your-jwt-token" \
  -H "Content-Type: application/json" \
  -d '{
    "template_name": "更新后的竞技场规则",
    "description": "更新了投票时间限制",
    "rules_config": {
      "game_flow": {
        "day_duration": 300,
        "night_duration": 180
      },
      "player_limits": {
        "max_players": 10,
        "min_players": 2
      },
      "voting": {
        "required_percentage": 60,
        "time_limit": 45
      }
    }
  }'
```

### 查询规则模版

```bash
# 查询所有激活的模版
curl "https://api.royale-arena.com/api/rule-templates?is_active=true"

# 按ID查询特定模版
curl "https://api.royale-arena.com/api/rule-templates?id=abc123-def456"

# 按名称搜索模版
curl "https://api.royale-arena.com/api/rule-templates?search=竞技场"
```

### 错误响应示例

```json
// 400 Bad Request - 参数验证失败
{
  "success": false,
  "error": {
    "message": "参数验证失败",
    "details": "模版名称不能为空"
  }
}

// 404 Not Found - 模版不存在
{
  "success": false,
  "error": {
    "message": "模版不存在",
    "details": "模版不存在"
  }
}

// 409 Conflict - 名称已存在
{
  "success": false,
  "error": {
    "message": "模版名称已存在",
    "details": "模版名称已存在"
  }
}
```

**Section sources**
- [handlers.rs](file://backend/src/rule_template/handlers.rs#L1-L66)
- [errors.rs](file://backend/src/rule_template/errors.rs#L1-L75)