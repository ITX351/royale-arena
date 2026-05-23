# 玩家行动调度优化设计

## 概述

本设计旨在解决玩家行动处理中重复验证逻辑的问题。当前每个玩家操作函数都需要在函数开头进行相同或相似的验证检查（如存活状态、体力值、出生状态等），导致代码重复且维护困难。通过引入独立的调度层来统一管理权限验证，实现验证逻辑与业务逻辑的解耦。

### 核心问题

- 每个玩家操作函数都包含大量重复的前置验证代码
- 不同操作需要不同的验证组合，无法简单地在消息处理层统一验证
- 验证逻辑散落在各个操作函数中，难以维护和扩展
- 体力消耗逻辑直接使用简单减法，不利于未来功能扩展

### 解决方案

创建独立的玩家行动调度模块和导演行动调度模块，在调度层根据行动类型执行差异化的权限验证，验证通过后调用纯业务逻辑函数。同时封装体力消耗函数，为未来扩展预留空间。

## 技术架构

### 调度层架构

```
graph TD
    A[WebSocket消息处理层] --> B{消息类型判断}
    B -->|玩家行动| C[PlayerActionScheduler::dispatch]
    B -->|导演行动| D[DirectorActionScheduler::dispatch]
    
    C --> E[权限验证层]
    E --> F{验证结果}
    F -->|通过| G[GameState业务逻辑]
    F -->|失败| H[返回错误ActionResult]
    
    D --> I[导演行为处理]
    I --> G
    
    G --> J[返回ActionResult]
    
    style C fill:#4CAF50
    style D fill:#2196F3
    style E fill:#FF9800
    style G fill:#9C27B0
```

### 模块职责划分

| 模块 | 职责 | 文件位置 |
|------|------|----------|
| WebSocket服务层 | 接收WebSocket消息，调用调度器 | `websocket/service.rs` |
| 玩家行动调度器 | 玩家行动权限验证与调度 | `websocket/player_action_scheduler.rs` |
| 导演行动调度器 | 导演行动调度（无需验证） | `websocket/director_action_scheduler.rs` |
| GameState业务逻辑 | 纯业务逻辑，不含权限验证 | `websocket/game_state_player_actions.rs`<br>`websocket/game_state_director_actions.rs` |

## 权限验证体系

### 权限需求分类

根据游戏规则和行动特性，定义以下权限验证类型：

| 权限类型 | 验证内容 | 适用场景 |
|---------|---------|---------|
| RequireAlive | 玩家必须存活（`is_alive == true`） | 除向导演发消息外的所有行动 |
| RequireBorn | 玩家必须已出生（`location`非空） | 除出生外的所有行动 |
| RequireStrength | 玩家体力必须满足最低要求（从rule_engine动态获取） | 消耗体力的行动 |
| RequireNotBound | 玩家未被捆绑（`is_bound == false`） | 除向导演发消息外的所有行动 |
| RequireInventorySpace | 背包必须有空闲空间（至少1格） | 仅拾取行动 |

**重要说明**：
- 所有体力验证必须从`rule_engine`动态获取消耗值，**绝对禁止硬编码体力值**
- 即使示例配置中某行动的体力消耗为0，也必须执行验证逻辑
- 生产环境的体力配置可能与示例不同，必须保证验证机制的完整性

### 玩家行动权限矩阵

| 行动类型 | 存活验证 | 已出生验证 | 体力验证 | 未捆绑验证 | 背包空间验证 | 备注 |
|---------|---------|-----------|---------|-----------|------------|------|
| born | ✓ | ✗ | ✗ | ✓ | - | 出生时验证未出生，不消耗体力 |
| move | ✓ | ✓ | ✓ | ✓ | - | 必须验证体力（从rule_engine获取move_cost） |
| search | ✓ | ✓ | ✓ | ✓ | - | 必须验证体力（从rule_engine获取search） |
| pick | ✓ | ✓ | ✓ | ✓ | ✓ | 必须验证体力（从rule_engine获取pick） |
| attack | ✓ | ✓ | ✓ | ✓ | - | 必须验证体力（从rule_engine获取attack） |
| equip | ✓ | ✓ | ✓ | ✓ | - | 必须验证体力（从rule_engine获取equip） |
| use | ✓ | ✓ | ✓ | ✓ | - | 必须验证体力（从rule_engine获取use_item） |
| throw | ✓ | ✓ | ✓ | ✓ | - | 必须验证体力（从rule_engine获取throw_item） |
| unequip | ✓ | ✓ | ✗ | ✓ | - | 卸下装备不消耗体力 |
| deliver | ✓ | ✓ | ✓ | ✓ | - | 必须验证体力（从rule_engine获取deliver） |
| send | ✗ | ✓ | ✗ | ✗ | - | 向导演发消息不消耗体力 |

**说明**：
- ✓ 表示需要该验证
- ✗ 表示不需要该验证
- \- 表示不相关
- **重要：所有需要体力验证的行动，必须从rule_engine动态获取体力消耗值，禁止硬编码**
- **警告：示例配置中某些行动的体力消耗为0，但这不意味着生产环境也为0，必须始终进行体力验证**

### 验证执行流程

```
sequenceDiagram
    participant WS as WebSocket服务层
    participant Scheduler as PlayerActionScheduler
    participant Validator as 权限验证函数
    participant GameState as GameState业务逻辑
    
    WS->>Scheduler: dispatch(action_type, params)
    Scheduler->>Scheduler: 获取行动权限配置
    
    alt 需要存活验证
        Scheduler->>Validator: check_player_alive()
        Validator-->>Scheduler: 验证结果
    end
    
    alt 需要出生验证
        Scheduler->>Validator: check_player_born()
        Validator-->>Scheduler: 验证结果
    end
    
    alt 需要体力验证
        Scheduler->>Validator: check_player_strength(required)
        Validator-->>Scheduler: 验证结果
    end
    
    alt 需要未捆绑验证
        Scheduler->>Validator: check_player_not_bound()
        Validator-->>Scheduler: 验证结果
    end
    
    alt 任一验证失败
        Scheduler-->>WS: 返回错误ActionResult
    else 所有验证通过
        Scheduler->>GameState: 调用业务逻辑函数
        GameState-->>Scheduler: 返回ActionResult
        Scheduler-->>WS: 返回结果
    end
```

## 调度器设计

### PlayerActionScheduler结构

PlayerActionScheduler是玩家行动调度的核心模块，负责权限验证和行动分发。

#### 主要职责

1. 根据行动类型确定所需权限验证项
2. 执行所有必需的权限验证
3. 验证通过后调用对应的GameState业务逻辑方法
4. 验证失败时返回友好的错误提示

#### 接口定义

```
PlayerActionScheduler::dispatch(
    game_state: &mut GameState,
    player_id: &str,
    action_type: &str,
    action_params: ActionParams
) -> Result<ActionResult, String>
```

**参数说明**：
- `game_state`: 可变游戏状态引用
- `player_id`: 执行行动的玩家ID
- `action_type`: 行动类型字符串（如"move", "attack"等）
- `action_params`: 行动参数结构体，包含各种可选参数

**返回值**：
- `Ok(ActionResult)`: 行动执行成功或验证失败（通过ActionResult传递错误信息）
- `Err(String)`: 系统级错误（如玩家不存在）

#### 行动参数结构

```
ActionParams {
    // 移动相关
    target_place: Option<String>,
    
    // 出生相关
    place_name: Option<String>,
    
    // 道具相关
    item_id: Option<String>,
    
    // 装备相关
    slot_type: Option<String>,
    
    // 消息相关
    target_player_id: Option<String>,
    message: Option<String>,
}
```

#### 权限验证函数

调度器内部提供以下私有验证函数：

| 函数名 | 功能 | 返回值 |
|--------|------|--------|
| `check_player_exists` | 验证玩家是否存在 | `Result<(), ActionResult>` |
| `check_player_alive` | 验证玩家是否存活 | `Result<(), ActionResult>` |
| `check_player_born` | 验证玩家是否已出生（location非空） | `Result<(), ActionResult>` |
| `check_player_strength` | 验证玩家体力是否充足 | `Result<(), ActionResult>` |
| `check_player_not_bound` | 验证玩家未被捆绑 | `Result<(), ActionResult>` |
| `check_inventory_space` | 验证背包有至少1格空间 | `Result<(), ActionResult>` |

**验证失败时返回**：
- 使用`ActionResult::new_info_message`创建错误结果
- 仅广播给发起者本人
- 提供中文友好错误提示
- 不向导演广播
- 不记录日志（Info类型消息不持久化）

### DirectorActionScheduler结构

DirectorActionScheduler负责导演行动的调度，由于导演拥有特殊权限，几乎不需要验证前置条件。

#### 主要职责

1. 接收导演行动请求
2. 直接调用对应的GameState导演业务逻辑方法
3. 返回处理结果

#### 接口定义

```
DirectorActionScheduler::dispatch(
    game_state: &mut GameState,
    action_type: &str,
    action_params: DirectorActionParams
) -> Result<ActionResult, String>
```

**参数说明**：
- `game_state`: 可变游戏状态引用
- `action_type`: 导演行动类型
- `action_params`: 导演行动参数

#### 导演行动参数结构

```
DirectorActionParams {
    // 时间设置
    timestamp: Option<String>,
    
    // 地点操作
    place_name: Option<String>,
    is_destroyed: Option<bool>,
    places: Option<Vec<String>>,
    
    // 天气操作
    weather: Option<f64>,
    
    // 玩家操作
    player_id: Option<String>,
    life_change: Option<i64>,
    strength_change: Option<i64>,
    target_place: Option<String>,
    action_type: Option<String>, // rope/unrope
    
    // 道具操作
    target_type: Option<String>,
    item: Option<Item>,
    
    // 消息操作
    message: Option<String>,
    
    // 批量操作
    airdrops: Option<Vec<AirdropItem>>,
    deletions: Option<Vec<ItemDeletionItem>>,
    clear_all: Option<bool>,
}
```

## GameState业务逻辑重构

### 移除验证逻辑

从`game_state_player_actions.rs`中所有玩家行动处理函数移除以下验证：

1. 移除`check_player_basic_status`函数调用
2. 移除`check_player_born_status`函数调用
3. 移除所有手动的存活状态检查
4. 移除所有手动的体力值检查
5. 移除所有手动的出生状态检查（location非空检查）
6. 删除`check_player_basic_status`和`check_player_born_status`函数定义

### 体力消耗封装

将所有直接的体力减法操作替换为调用统一的体力消耗函数。

#### 体力消耗函数定义

```
GameState::consume_strength(
    player_id: &str,
    amount: i32
) -> Result<(), String>
```

**功能**：
- 从指定玩家的当前体力中扣除指定数量
- 确保体力不低于0
- 为未来扩展预留（如装备影响、buff效果等）

**当前实现**：
- 简单减法：`player.strength -= amount`
- 边界检查：`if player.strength < 0 { player.strength = 0; }`

**未来扩展可能**：
- 装备减免消耗效果
- 特殊状态下的消耗加成/减免
- 消耗统计和分析
- 体力透支机制

#### 体力消耗位置

所有玩家行动处理函数中的体力消耗均应调用`consume_strength`：

- `handle_move_action`: 消耗移动体力
- `handle_search_action`: 消耗搜索体力
- `handle_pick_action`: 消耗拾取体力
- `handle_attack_action`: 消耗攻击体力
- `handle_equip_action`: 消耗装备体力
- `handle_use_action`: 消耗使用体力
- `handle_throw_action`: 消耗丢弃体力
- `handle_deliver_action`: 消耗传音体力

### 辅助验证函数处理

原有的`check_player_basic_status`和`check_player_born_status`函数将被删除，因为它们的职责已转移到调度层。

**注意**：
- `is_born`字段将被删除，不再使用
- 所有判断玩家是否已出生的逻辑改为检查`location`是否为空
- `location`为空表示未出生，`location`非空表示已出生

## WebSocket服务层集成

### 修改点

在`websocket/service.rs`中：

1. 将`process_player_action`函数中的行动处理逻辑替换为调用`PlayerActionScheduler::dispatch`
2. 将`process_director_action`函数中的行动处理逻辑替换为调用`DirectorActionScheduler::dispatch`
3. 移除原有的参数解析和分发逻辑，统一由调度器处理

### 调用示例

**玩家行动处理**：

```
// 原有代码
match action {
    "move" => {
        let target_place = action_data.get("target_place")...;
        game_state.handle_move_action(player_id, target_place)
    }
    ...
}

// 新代码
let action_params = ActionParams::from_json(action_data)?;
PlayerActionScheduler::dispatch(
    &mut game_state,
    player_id,
    action,
    action_params
)
```

**导演行动处理**：

```
// 原有代码
match action {
    "weather" => {
        let weather = action_data.get("weather")...;
        game_state.handle_weather(weather)
    }
    ...
}

// 新代码
let action_params = DirectorActionParams::from_json(action_data)?;
DirectorActionScheduler::dispatch(
    &mut game_state,
    action,
    action_params
)
```

## 数据流设计

### 完整数据流图

```
flowchart TD
    A[WebSocket客户端消息] --> B[WebSocket服务层]
    
    B --> C{消息类型}
    
    C -->|PlayerAction| D[构造ActionParams]
    C -->|DirectorAction| E[构造DirectorActionParams]
    
    D --> F[PlayerActionScheduler::dispatch]
    E --> G[DirectorActionScheduler::dispatch]
    
    F --> H{行动类型查表}
    H --> I[执行权限验证链]
    
    I --> J{验证通过?}
    J -->|否| K[返回Info类型ActionResult]
    J -->|是| L[调用GameState业务函数]
    
    L --> M[执行业务逻辑]
    M --> N{需要消耗体力?}
    N -->|是| O[调用consume_strength]
    N -->|否| P[直接操作状态]
    
    O --> Q[返回ActionResult]
    P --> Q
    
    G --> R[调用GameState导演函数]
    R --> S[执行导演操作]
    S --> T[返回ActionResult]
    
    K --> U[回传WebSocket服务层]
    Q --> U
    T --> U
    
    U --> V[广播到相关玩家]
    U --> W[广播到导演可选]
    U --> X[记录日志可选]
    
    style F fill:#4CAF50
    style G fill:#2196F3
    style I fill:#FF9800
    style M fill:#9C27B0
    style O fill:#E91E63
```

## 错误处理设计

### 错误类型

| 错误类型 | 处理方式 | 返回格式 |
|---------|---------|---------|
| 权限验证失败 | 返回Info类型ActionResult | 不记录日志，仅通知玩家 |
| 玩家不存在 | 返回Err(String) | 系统级错误 |
| 参数缺失/格式错误 | 返回Err(String) | 系统级错误 |
| 业务逻辑错误 | 返回Info类型ActionResult | 由GameState函数决定 |

### 错误提示信息规范

所有错误提示必须使用简体中文，并提供清晰的错误原因：

| 验证失败类型 | 错误提示示例 |
|-------------|-------------|
| 玩家未找到 | "玩家未找到" |
| 玩家已死亡 | "玩家已死亡，无法进行操作" |
| 玩家未出生 | "玩家尚未出生，请先选择出生地点" |
| 玩家已出生 | "玩家已经出生，无法重复出生" |
| 体力不足 | "体力不足，无法执行该操作" |
| 玩家被捆绑 | "玩家被捆绑，无法自由行动" |
| 背包已满 | "背包已满，无法拾取更多物品" |

## 文件结构

### 新增文件

```
backend/src/websocket/
├── player_action_scheduler.rs    // 玩家行动调度器
├── director_action_scheduler.rs  // 导演行动调度器
```

### 修改文件

```
backend/src/websocket/
├── service.rs                          // 调用调度器
├── game_state_player_actions.rs        // 移除验证逻辑，添加consume_strength
├── game_state_director_actions.rs      // 保持不变（如需要）
├── models.rs                           // 添加ActionParams和DirectorActionParams
```

### 模块声明

在`backend/src/websocket.rs`中添加模块声明：

```
pub mod player_action_scheduler;
pub mod director_action_scheduler;
```

## 向后兼容性

**本次修改为破坏性更改**，不考虑向后兼容。主要影响：

1. `GameState`中的`handle_*_action`函数签名保持不变，但内部验证逻辑被移除
2. WebSocket服务层的调用方式完全改变
3. 所有直接调用GameState行动处理函数的代码需要改为调用调度器
4. 体力消耗逻辑从直接减法改为函数调用

## 实现范围

### 包含内容

1. 创建`PlayerActionScheduler`模块，实现所有玩家行动的权限验证和调度
2. 创建`DirectorActionScheduler`模块，实现所有导演行动的调度
3. 定义`ActionParams`和`DirectorActionParams`参数结构
4. 在`GameState`中实现`consume_strength`体力消耗函数
5. 从`game_state_player_actions.rs`移除所有验证逻辑
6. 将所有体力减法替换为`consume_strength`调用
7. 修改`websocket/service.rs`调用调度器
8. 删除`check_player_basic_status`和`check_player_born_status`函数
9. 从 Player 模型中删除`is_born`字段，使用`location`判断出生状态

### 不包含内容

1. 不编写任何测试代码
2. 不修改前端代码
3. 不添加日志记录功能（使用现有机制）
4. 不修改数据库相关代码
5. 不优化现有业务逻辑（仅重构验证部分）

## 实施步骤

### 第一阶段：创建调度器基础结构

1. 创建`player_action_scheduler.rs`文件
2. 定义`ActionParams`结构体
3. 实现`PlayerActionScheduler::dispatch`函数框架
4. 实现所有权限验证辅助函数

### 第二阶段：实现玩家行动调度

1. 根据权限矩阵，为每种玩家行动配置验证需求
2. 在`dispatch`函数中实现行动类型分发
3. 每个行动分支调用对应的验证链
4. 验证通过后调用GameState业务函数

### 第三阶段：创建导演调度器

1. 创建`director_action_scheduler.rs`文件
2. 定义`DirectorActionParams`结构体
3. 实现`DirectorActionScheduler::dispatch`函数
4. 为每种导演行动建立分发路由

### 第四阶段：重构GameState业务逻辑

1. 实现`GameState::consume_strength`函数
2. 从所有玩家行动函数移除验证逻辑
3. 替换所有体力减法为`consume_strength`调用
4. 删除`check_player_basic_status`和`check_player_born_status`
5. 从 Player 结构体删除`is_born`字段
6. 修改所有判断出生状态的逻辑为检查`location`是否为空

### 第五阶段：集成WebSocket服务层

1. 修改`process_player_action`调用`PlayerActionScheduler`
2. 修改`process_director_action`调用`DirectorActionScheduler`
3. 调整参数解析逻辑
4. 移除原有的行动类型match分支

### 第六阶段：验证和调整

1. 检查所有行动类型是否正确映射
2. 验证错误提示信息是否友好
3. 确认体力消耗逻辑正确
4. 确认广播逻辑未被破坏

## 权限验证详细规范

### 验证函数实现规范

每个验证函数应：

1. 接收`GameState`和`player_id`作为参数
2. 从`GameState`中获取玩家信息
3. 执行单一验证职责
4. 验证失败时返回`Err(ActionResult::new_info_message(...))`
5. 验证成功时返回`Ok(())`

### 验证链执行规范

调度器执行验证时应：

1. 按照固定顺序执行验证（存活 → 出生 → 体力 → 其他）
2. 使用`?`操作符提前返回验证失败
3. 所有验证通过后才调用业务逻辑
4. 不重复执行相同验证

### 特殊验证场景

**出生行动特殊处理**：
- 出生行动需要验证玩家存活但不需要已出生
- 验证玩家尚未出生（location为空）
- 这是唯一一个验证"未出生"的行动

**攻击行动特殊处理**：
- 体力消耗在调度层验证为0
- 实际体力消耗在业务逻辑中处理（根据是否有武器）
- 目标玩家的验证在业务逻辑中完成

**拾取行动特殊处理**：
- 背包空间验证在调度层完成
- 需要至少1格空闲空间（总空间 - 当前物品数 >= 1）
- 背包空间计数包括已装备的武器和防具

**向导演发消息特殊处理**：
- 不需要存活验证（已死亡玩家可以发消息）
- 不需要未捆绑验证
- 仅需要已出生验证

## 体力消耗函数设计细节

### 函数签名

```
impl GameState {
    pub fn consume_strength(&mut self, player_id: &str, amount: i32) -> Result<(), String>
}
```

### 实现逻辑

1. 获取玩家可变引用
2. 执行减法：`player.strength -= amount`
3. 边界检查：`if player.strength < 0 { player.strength = 0; }`
4. 返回`Ok(())`

### 未来扩展预留

函数内部可以后续添加：

- 装备减免效果检查
- 特殊buff状态检查
- 消耗记录统计
- 体力透支标记
- 消耗事件通知

### 调用位置

体力消耗应在业务逻辑处理的最后阶段调用，确保行动确实会执行才消耗体力。

## 消息类型和广播规则

### ActionResult类型使用

| 场景 | 使用方法 | 广播范围 | 是否记录日志 |
|------|---------|---------|-------------|
| 权限验证失败 | `ActionResult::new_info_message` | 仅发起者 | 否（Info类型不持久化） |
| 业务逻辑错误 | `ActionResult::new_info_message` | 仅发起者 | 否 |
| 系统通知消息 | `ActionResult::new_system_message` | 相关玩家 | 是 |
| 用户定向消息 | `ActionResult::new_user_message` | 相关玩家 | 是 |

### 广播策略

权限验证失败时：
- `broadcast_players`: 仅包含发起者
- `broadcast_to_director`: false
- 不影响其他玩家和导演

业务逻辑执行后：
- 由GameState函数决定广播范围
- 保持现有广播逻辑不变