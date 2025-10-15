# 导演端游戏编辑API设计文档

## 概述

本设计文档定义导演端游戏编辑功能，该功能允许导演在不使用管理员JWT令牌的情况下，通过导演密码验证来修改游戏基本属性（游戏名称、描述和规则配置）。此功能与现有的管理员游戏更新接口平行，但专为导演操作场景设计。

### 设计目标

- 为导演提供独立的游戏编辑能力，无需依赖管理员权限
- 仅通过导演密码验证，不涉及JWT认证流程
- 移除密码编辑功能以保障安全性
- 破坏性修改，不考虑向后兼容
- 最小化实现，避免过度设计

### 核心价值

- **权限分离**：导演操作与管理员操作逻辑隔离，职责清晰
- **安全性**：禁止导演修改自身密码，防止权限滥用
- **简化流程**：规则编辑直接调用导演接口，修复前端调用管理员接口无法正确运行的问题

## 架构设计

### 系统边界

```
graph TB
    subgraph "前端导演控制台"
        A[PreGameManagement]
        B[RuleManagement]
        C[游戏属性编辑对话框]
    end
    
    subgraph "后端导演服务"
        D[DirectorHandler]
        E[DirectorService]
        F[导演密码验证]
    end
    
    subgraph "数据层"
        G[Games表]
    end
    
    A --> C
    B --> D
    C --> D
    D --> F
    F --> E
    E --> G
```

### 与现有功能的关系

| 功能特性 | 管理员游戏更新接口 | 导演游戏编辑接口 |
|---------|-----------------|---------------|
| 认证方式 | JWT令牌 | 导演密码（查询参数） |
| 路由前缀 | `/api/admin/games/{game_id}` | `/api/game/{game_id}/edit` |
| 可编辑字段 | 名称、描述、密码、最大玩家数、规则配置 | 名称、描述、规则配置 |
| 前置权限校验 | JWT中间件 + 管理员权限 | 导演密码验证 |
| 业务逻辑位置 | GameService | DirectorService |

## API端点设计

### 导演编辑游戏接口

**端点**：`PUT /api/game/{game_id}/edit`

**认证方式**：导演密码（查询参数）

**用途**：导演修改游戏的基本属性（名称、描述、规则配置）

#### 请求规范

| 参数位置 | 参数名 | 类型 | 必填 | 说明 |
|---------|--------|------|------|------|
| 路径参数 | game_id | String | 是 | 游戏唯一标识符 |
| 查询参数 | password | String | 是 | 导演密码 |
| 请求体 | name | String | 否 | 游戏名称（最大100字符） |
| 请求体 | description | String | 否 | 游戏描述 |
| 请求体 | max_players | Integer | 否 | 最大玩家数（1-1000） |
| 请求体 | rules_config | JSON Object | 否 | 游戏规则配置JSON |

#### 请求体示例

```
{
  "name": "新的游戏名称",
  "description": "更新后的游戏描述",
  "max_players": 100,
  "rules_config": {
    "map": {
      "places": ["地点A", "地点B"],
      "safe_places": ["地点A"]
    },
    "player": {
      "max_life": 100,
      "max_strength": 10
    }
  }
}
```

#### 响应规范

**成功响应（200 OK）**

```
{
  "success": true,
  "data": {
    "id": "game-uuid",
    "name": "新的游戏名称",
    "description": "更新后的游戏描述",
    "status": "waiting",
    "max_players": 50,
    "player_count": 0,
    "rules_config": { ... },
    "created_at": "2025-01-15T10:00:00Z",
    "updated_at": "2025-01-15T12:00:00Z"
  }
}
```

**错误响应**

| HTTP状态码 | 场景 | 响应体示例 |
|-----------|------|-----------|
| 400 | 请求参数验证失败 | `{"success": false, "error": "游戏名称不能为空"}` |
| 401 | 导演密码错误 | `{"success": false, "error": "导演密码错误"}` |
| 404 | 游戏不存在 | `{"success": false, "error": "游戏不存在"}` |
| 500 | 服务器内部错误 | `{"success": false, "error": "服务器错误"}` |

#### 业务规则

1. **密码验证优先**：在执行任何数据更新前，必须验证导演密码的正确性
2. **部分更新语义**：所有字段均为可选，仅更新请求中包含的字段
3. **禁止密码修改**：请求模型中不包含 `director_password` 字段，导演无法修改自身密码
4. **数据验证**：
   - 游戏名称：非空且长度不超过100字符
   - 最大玩家数：1-1000之间的整数
   - 规则配置：必须是合法的JSON对象
5. **状态无关**：无论游戏处于何种状态（waiting/running/paused/ended），均允许编辑

### 路由注册

**位置**：`backend/src/routes.rs`

**路由组**：导演路由组（无JWT认证，使用导演密码验证）

**新增路由项**：

```
PUT /api/game/{game_id}/edit -> director::handlers::edit_game
```

## 数据模型设计

### 导演编辑游戏请求模型

**模型名称**：`DirectorEditGameRequest`

**定义位置**：`backend/src/director/models.rs`

**字段定义**

| 字段名 | 数据类型 | 可选性 | 默认值 | 说明 |
|--------|---------|--------|-------|------|
| name | String | 可选 | - | 游戏名称 |
| description | String | 可选 | - | 游戏描述 |
| max_players | i32 | 可选 | - | 最大玩家数 |
| rules_config | JSON Value | 可选 | - | 规则配置对象 |

**验证逻辑**

- `name` 存在时：
  - 去除前后空格后不能为空
  - 长度不超过100字符
- `max_players` 存在时：
  - 必须在1-1000之间
- `rules_config` 存在时：
  - 必须是有效的JSON对象（序列化验证由serde自动完成）
- 至少提供一个字段，否则返回验证错误

**与 `UpdateGameRequest` 的差异**

| 字段 | UpdateGameRequest | DirectorEditGameRequest |
|------|------------------|------------------------|
| name | 可选 | 可选 |
| description | 可选 | 可选 |
| director_password | 可选 | **不存在** |
| max_players | 可选 | 可选 |
| rules_config | 可选 | 可选 |

## 业务逻辑层设计

### DirectorService 新增方法

**方法签名**：`edit_game`

**方法职责**：
1. 验证导演密码
2. 验证请求参数
3. 构造数据库更新语句（仅更新提供的字段）
4. 执行更新操作
5. 返回更新后的游戏信息

**执行流程**

```
sequenceDiagram
    participant H as Handler
    participant S as DirectorService
    participant DB as Database
    
    H->>S: edit_game(game_id, password, request)
    S->>DB: 验证导演密码
    alt 密码错误
        DB-->>S: 错误
        S-->>H: DirectorError::Unauthorized
    end
    
    S->>S: 验证请求参数
    alt 验证失败
        S-->>H: DirectorError::ValidationError
    end
    
    S->>DB: 构建动态UPDATE语句
    S->>DB: 执行更新
    alt 游戏不存在
        DB-->>S: 0行受影响
        S-->>H: DirectorError::GameNotFound
    end
    
    S->>DB: 查询更新后的游戏
    DB-->>S: GameWithPlayerCounts
    S-->>H: 返回游戏数据
```

**错误处理**

| 错误类型 | 触发条件 | HTTP状态码 |
|---------|---------|-----------|
| Unauthorized | 导演密码验证失败 | 401 |
| ValidationError | 请求参数验证失败 | 400 |
| GameNotFound | 游戏ID不存在 | 404 |
| DatabaseError | 数据库操作失败 | 500 |

### 数据库操作策略

**动态SQL构建**

由于字段为可选，需要根据请求中提供的字段动态构建UPDATE语句：

- 如果 `name` 存在，添加 `name = ?`
- 如果 `description` 存在，添加 `description = ?`
- 如果 `max_players` 存在，添加 `max_players = ?`
- 如果 `rules_config` 存在，添加 `rules_config = ?`
- 自动更新 `updated_at` 字段为当前时间

**SQL示例**

```
-- 全字段更新
UPDATE games 
SET name = ?, description = ?, max_players = ?, rules_config = ?, updated_at = NOW()
WHERE id = ?;

-- 仅更新名称
UPDATE games 
SET name = ?, updated_at = NOW()
WHERE id = ?;
```

**事务处理**：单个UPDATE语句，无需显式事务

## 前端集成设计

### RuleManagement组件修改

**修改文件**：`frontend/src/views/director/management/RuleManagement.vue`

**修改范围**：`saveRules` 方法

**当前实现问题**

- 调用管理员接口 `gameService.updateGame()`
- 需要JWT令牌认证
- 在导演页面无法正确运行（无JWT上下文）

**目标实现**

- 调用导演专用接口 `directorService.editGame()`
- 使用导演密码认证（从 props 获取）
- 仅更新 `rules_config` 字段

**方法调用变更**

| 修改前 | 修改后 |
|--------|--------|
| `gameService.updateGame(game.id, { rules_config })` | `directorService.editGame(game.id, directorPassword, { rules_config })` |

### PreGameManagement组件增强

**修改文件**：`frontend/src/views/director/management/PreGameManagement.vue`

**新增功能**：游戏属性编辑对话框

#### 对话框UI设计

**触发方式**：在页面顶部增加"编辑游戏属性"按钮

**对话框内容**

| 字段 | 输入类型 | 验证规则 |
|------|---------|---------|
| 游戏名称 | 文本输入框 | 非空，最大100字符 |
| 游戏描述 | 多行文本输入框 | 可选，无长度限制 |

**说明**：虽然后端支持编辑最大玩家数和规则配置，但前端当前版本仅实现名称和描述的编辑。最大玩家数可在后续版本中添加编辑功能。规则配置的编辑已由 RuleManagement 组件处理。

**按钮操作**

- **保存**：调用 `directorService.editGame()`，成功后触发 `refresh` 事件
- **取消**：关闭对话框，不保存修改

#### 对话框交互流程

```
stateDiagram-v2
    [*] --> 对话框关闭
    对话框关闭 --> 对话框打开: 点击编辑按钮
    对话框打开 --> 对话框关闭: 点击取消
    对话框打开 --> 保存中: 点击保存
    保存中 --> 对话框关闭: 保存成功
    保存中 --> 对话框打开: 保存失败（显示错误）
```

#### 数据绑定

- **初始值**：从 `props.game` 读取 `name` 和 `description`
- **修改状态检测**：比较当前值与初始值，仅当存在变更时启用保存按钮
- **保存后操作**：调用 `emit('refresh')` 通知父组件刷新游戏数据

### API服务层

**新增文件**：`frontend/src/services/directorService.ts`（如不存在）

**新增方法**：`editGame`

**方法签名**

```
editGame(
  gameId: string, 
  directorPassword: string, 
  data: {
    name?: string
    description?: string
    max_players?: number
    rules_config?: any
  }
): Promise<ApiResponse<GameWithRules>>
```

**实现要点**

- HTTP方法：PUT
- 路径：`/api/game/${gameId}/edit`
- 查询参数：`?password=${directorPassword}`
- 请求体：JSON格式的 `data` 对象

## 集成测试设计

### 测试策略

**测试文件**：`backend/tests/director_integration.rs`

**测试方法**：集成到现有的 `test_director_comprehensive_integration` 函数中

**测试覆盖范围**

| 测试编号 | 测试场景 | 预期结果 |
|---------|---------|---------|
| T1 | 使用错误的导演密码编辑游戏 | 返回401错误 |
| T2 | 编辑游戏名称（有效输入） | 成功更新，返回新名称 |
| T3 | 编辑游戏描述（有效输入） | 成功更新，返回新描述 |
| T4 | 编辑最大玩家数（有效输入） | 成功更新，返回新玩家数 |
| T5 | 同时编辑名称、描述和最大玩家数 | 所有字段均成功更新 |
| T6 | 编辑规则配置 | 成功更新，返回新规则 |
| T7 | 提供空名称 | 返回400验证错误 |
| T8 | 提供超长名称（101字符） | 返回400验证错误 |
| T9 | 提供无效的最大玩家数（0或1001） | 返回400验证错误 |
| T10 | 编辑不存在的游戏 | 返回404错误 |
| T11 | 请求体为空（无任何字段） | 返回400验证错误或接受请求（实现决策） |

### 测试数据准备

**利用现有测试数据**

- 游戏ID：由 `test_director_comprehensive_integration` 创建的 `game_id`
- 导演密码：`"test123"`
- 错误密码：`"wrong_password"`

**测试插入位置**

在现有测试的"测试7"（数据验证）之后，"测试8"（新增）

### 测试断言要点

1. **密码验证**：确认使用错误密码时返回 `DirectorError::Unauthorized`
2. **字段更新**：通过查询验证字段值已正确更新
3. **部分更新**：仅更新提供的字段，其他字段保持不变
4. **时间戳**：验证 `updated_at` 字段已更新
5. **返回数据完整性**：确认响应包含完整的游戏信息

## 实现清单

### 后端实现

| 任务 | 文件 | 说明 |
|------|------|------|
| 定义请求模型 | `backend/src/director/models.rs` | 添加 `DirectorEditGameRequest` 结构体及验证方法 |
| 实现处理函数 | `backend/src/director/handlers.rs` | 添加 `edit_game` 处理函数 |
| 实现业务逻辑 | `backend/src/director/service.rs` | 添加 `edit_game` 服务方法 |
| 注册路由 | `backend/src/routes.rs` | 在导演路由组添加 `PUT /api/game/{game_id}/edit` |
| 添加集成测试 | `backend/tests/director_integration.rs` | 在现有测试函数中添加T1-T11测试用例 |

### 前端实现

| 任务 | 文件 | 说明 |
|------|------|------|
| 创建/更新API服务 | `frontend/src/services/directorService.ts` | 添加 `editGame` 方法 |
| 修改规则保存逻辑 | `frontend/src/views/director/management/RuleManagement.vue` | 替换 `saveRules` 中的 API 调用 |
| 添加属性编辑对话框 | `frontend/src/views/director/management/PreGameManagement.vue` | 新增对话框组件和编辑逻辑 |
| 类型定义 | `frontend/src/types/game.ts` | 确保类型定义支持新接口（如已存在则无需修改） |

## 数据流图

### 规则编辑流程

```
sequenceDiagram
    participant U as 导演用户
    participant R as RuleManagement
    participant D as DirectorService
    participant B as Backend API
    participant DB as Database
    
    U->>R: 编辑规则JSON
    U->>R: 点击保存
    R->>R: 验证JSON格式
    R->>D: editGame(gameId, password, {rules_config})
    D->>B: PUT /api/game/{id}/edit?password=xxx
    B->>DB: 验证导演密码
    alt 密码错误
        DB-->>B: 验证失败
        B-->>D: 401错误
        D-->>R: 错误响应
        R->>U: 显示错误消息
    else 密码正确
        DB-->>B: 验证成功
        B->>DB: UPDATE games SET rules_config=...
        DB-->>B: 更新成功
        B-->>D: 游戏数据
        D-->>R: 成功响应
        R->>R: emit('refresh')
        R->>U: 显示成功消息
    end
```

### 游戏属性编辑流程

```
sequenceDiagram
    participant U as 导演用户
    participant P as PreGameManagement
    participant D as DirectorService
    participant B as Backend API
    
    U->>P: 点击"编辑游戏属性"
    P->>U: 显示编辑对话框
    U->>P: 修改名称/描述
    U->>P: 点击保存
    P->>D: editGame(gameId, password, {name, description})
    D->>B: PUT /api/game/{id}/edit?password=xxx
    B-->>D: 成功响应
    D-->>P: 游戏数据
    P->>P: emit('refresh')
    P->>U: 关闭对话框，显示成功消息
```

## 安全性考量

### 密码保护策略

- **禁止密码修改**：导演无法通过此接口修改 `director_password`，防止权限篡改
- **密码传输**：通过查询参数传递（与现有导演接口一致），建议生产环境使用HTTPS

### 权限边界

- **导演权限范围**：可编辑游戏名称、描述、最大玩家数和规则配置，但不涉及：
  - 游戏删除（管理员专属）
  - 导演密码修改（管理员专属）
- **状态限制**：无状态限制，但需注意游戏运行中修改规则或最大玩家数可能导致逻辑冲突（业务层需额外处理）

## 错误处理规范

### 错误响应格式

所有错误响应遵循统一格式：

```
{
  "success": false,
  "error": "错误描述信息"
}
```

### 前端错误处理

1. **网络错误**：显示通用错误提示"网络请求失败，请稍后重试"
2. **401错误**：提示"导演密码错误，请检查密码"
3. **400错误**：显示服务器返回的具体验证错误信息
4. **404错误**：提示"游戏不存在或已被删除"
5. **500错误**：提示"服务器错误，请稍后重试"

### 后端错误日志

- 记录所有数据库错误的详细堆栈信息
- 记录导演密码验证失败的游戏ID和尝试时间（审计用途）
- 不记录导演密码的明文内容

## 非功能性需求

### 性能要求

- **响应时间**：正常情况下编辑操作应在500ms内完成
- **并发处理**：支持同一游戏的并发编辑请求，使用数据库行锁保证数据一致性

### 可维护性

- **代码复用**：复用 `GameService` 的查询方法获取更新后的游戏数据
- **错误类型一致**：使用现有的 `DirectorError` 枚举类型
- **验证逻辑集中**：将验证逻辑封装在请求模型的 `validate` 方法中

### 兼容性

- **破坏性变更声明**：本功能为破坏性修改，不保证向后兼容
- **前端API版本**：前端直接调用新接口，移除对管理员接口的依赖
- **数据库结构**：无需修改数据库表结构，使用现有 `games` 表

## 依赖关系

### 后端依赖

- `DirectorService::verify_director_password`：验证导演密码
- `GameService::get_game_by_id_with_player_counts`：获取完整游戏信息
- `sqlx::MySqlPool`：数据库连接池

### 前端依赖

- `directorPassword` 来自父组件 props（已存在）
- `gameService` 类型定义（复用现有类型）
- Element Plus 组件库（对话框、表单、消息提示）

## 部署注意事项

### 后端部署

1. 确保新路由已注册到 `routes.rs` 的导演路由组
2. 重新编译并运行集成测试，确保所有测试通过
3. 验证API文档已更新（如使用OpenAPI生成工具）

### 前端部署

1. 确认 `directorService.ts` 文件已创建或更新
2. 检查 RuleManagement 组件的导入路径正确性
3. 清除浏览器缓存以加载新前端资源

### 测试验证清单

- [ ] 后端集成测试全部通过
- [ ] 前端编译无错误
- [ ] 手动测试规则编辑功能
- [ ] 手动测试游戏属性编辑对话框
- [ ] 验证错误场景的提示信息准确性
- [ ] 检查浏览器控制台无JavaScript错误
