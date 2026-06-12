# Royale Arena 代码修改总结报告

> 对比基准：`royale-arena-main_v1/`（初版） vs 当前代码
> 生成日期：2026-05-22

---

## 一、修改总览

本次修改共涉及 **20 个源文件**（后端 9 个，前端 11 个），涵盖三大功能方向：

| 功能方向 | 后端文件 | 前端文件 | 说明 |
|---------|---------|---------|------|
| 货币（金币）系统 | 6 | 8 | 全链路新增货币类型与玩家货币字段 |
| 导演移动玩家 | 3 | 1 | 导演界面可直接选择玩家位置 |
| 编译/构建修复 | 3 | 0 | SQL DateTime 类型注解 + Windows jemalloc 兼容 |

---

## 二、后端修改详情（Rust）

### 2.1 新增货币系统

#### 2.1.1 `game/game_rule_engine.rs` — 玩家模型新增货币字段

```rust
// Player 结构体新增字段
pub struct Player {
    // ... 原有字段
    /// 货币总数
    #[serde(default)]
    pub coins: i32,  // 新增
}
```

同时在 `Default` 实现中初始化为 `coins: 0`。

#### 2.1.2 `game/game_rule_engine.rs` — 物品类型新增货币

```rust
// ItemType 枚举新增变体
pub enum ItemType {
    Weapon(WeaponProperties),
    Armor(ArmorProperties),
    Consumable(ConsumableProperties),
    Utility(UtilityProperties),
    Upgrader,
    Currency(CurrencyProperties),  // 新增
}

// 新增货币属性结构体
pub struct CurrencyProperties {
    pub value: i32,
}

// 新增货币配置结构体
pub struct CurrencyConfig {
    pub name: String,
    pub internal_name: Option<String>,
    pub rarity: Option<String>,
    pub properties: CurrencyProperties,
}

// ItemsByCategory 新增 currencies 字段
pub struct ItemsByCategory {
    pub weapons: Vec<WeaponConfig>,
    pub armors: Vec<ArmorConfig>,
    pub utilities: Vec<UtilityConfig>,
    pub consumables: Vec<ConsumableConfig>,
    pub upgraders: Vec<UpgraderConfig>,
    pub currencies: Vec<CurrencyConfig>,  // 新增
}
```

在 `create_item_from_name` 方法中新增货币搜索分支（第 6 优先级），通过 `name` 匹配并创建 `ItemType::Currency` 物品实例。

#### 2.1.3 `websocket/actions/player_use_action.rs` — 货币使用逻辑

新增 `handle_currency_use` 方法，当玩家"使用"货币物品时：
- 将货币的 `value` 累加到玩家的 `coins` 字段
- 物品被消耗（`reinsert = false`）
- 返回提示消息 `"使用 {name} 获得 {value} 货币"`

```rust
// 新增导入
use crate::game::game_rule_engine::{ConsumableProperties, CurrencyProperties, Item, ItemType, UtilityProperties};

// handle_use_item 的 match 中新增分支
ItemType::Currency(properties) => self.handle_currency_use(
    player_id, &player_name, &item.name, properties,
    strength_before, use_cost,
),
```

#### 2.1.4 `websocket/actions/game_state_common.rs` — 死亡货币处理逻辑重构

**核心逻辑变更**：`kill_player` 方法中货币处理从依赖 `death_item_disposition` 配置改为按死因分类处理。

**初版逻辑**：
```rust
// 货币转移完全依赖 death_item_disposition 配置
if victim_coins > 0 {
    if matches!(disposition, DeathDisposition::KillerTakes) {
        if let Some(loot_player_id) = loot_recipient_id {
            // 击杀者获得货币
        }
    }
    // 其他情况下货币静默消失，无日志
}
```

**当前逻辑**：
```rust
// 货币处理：仅被玩家攻击致死时击杀者缴获全部货币，其余死因货币直接消失
if victim_coins > 0 {
    if let Some(loot_player_id) = loot_recipient_id {
        // PVP 击杀：击杀者获得全部货币
        killer.coins += victim_coins;
        transferred_coins = victim_coins;
    }
    // 非 PVP 击杀：货币直接消失（victim_coins 已在上方清零）
}
```

**日志增强**：
```rust
// PVP 击杀：显示缴获货币
if transferred_coins > 0 {
    detail_segments.push(format!("缴获货币: {}", transferred_coins));
}
// 非 PVP 死亡：显示消失货币
if victim_coins > 0 && transferred_coins == 0 {
    detail_segments.push(format!("消失货币: {}", victim_coins));
}
```

**变更要点**：
- `kill_player` 方法的元组解构从 `(mut loot_items, previous_location)` 扩展为 `(mut loot_items, previous_location, victim_coins)`
- 玩家死亡时 `player.coins = 0` 立即清零
- 新增 `transferred_coins: i32` 追踪变量
- JSON 返回数据新增 `transferred_coins` 字段

#### 2.1.5 `websocket/actions/director_common_actions.rs` — 导演设置货币

新增 `handle_set_player_coins` 方法，与已有的 `handle_set_player_life` / `handle_set_player_strength` 遵循相同模式：
- 查找玩家 → 检查值是否变化 → 更新值 → 返回 ActionResult
- 支持增量日志显示（`format_delta` 格式化正负号）

### 2.2 新增导演移动玩家功能

#### 2.2.1 `websocket/actions/director_common_actions.rs` — 移动逻辑实现

新增 `handle_move_player` 方法，完整流程：

```
1. 查找玩家，获取当前位置和名称
2. 若目标位置与当前位置相同 → 返回"未变化"提示
3. 验证目标地点是否存在 → 不存在则报错
4. 验证目标地点是否被摧毁 → 摧毁则返回失败
5. 从旧地点的 players 列表移除该玩家 ID
6. 更新 player.location = target_place
7. 将玩家 ID 添加到新地点的 players 列表
8. 返回 ActionResult（包含 player_id + 新 location）
```

此方法与玩家自行移动（`handle_move_action`）的区别：
- **不消耗体力**：导演移动是管理操作，不扣减 strength
- **不清理搜索结果**：不调用 `daily_reset`
- **不检查距离/安全区**：可以传送到任何有效地点

#### 2.2.2 `websocket/actions/director_action_scheduler.rs` — 调度注册

在 dispatch match 表中新增两个 action：

```rust
// 导演设置货币
"coins" => { ... game_state.handle_set_player_coins(...) }

// 导演移动玩家（复用已有的 target_place 字段）
"move_player" => { ... game_state.handle_move_player(...) }
```

`DirectorActionParams` 结构体的 `target_place` 字段原先未被任何 action 使用，现在被 `move_player` action 使用。

### 2.3 编译/构建修复

#### 2.3.1 `game/log_service.rs` — SQL DateTime 类型注解

**问题**：`sqlx::query_as!` 宏在编译时从 MySQL 元数据推断列类型，部分 `NOT NULL` 的 `DATETIME` 列被推断为 `Option<DateTime<Utc>>`，但 Rust 结构体字段类型为 `DateTime<Utc>`，导致类型不匹配编译错误。

**修复方式**：在所有受影响的 SELECT 语句中添加 `sqlx` 强制非空类型注解：

```sql
-- 修复前
timestamp, kill_time, created_at, updated_at

-- 修复后（使用 ! 强制非空 + 显式类型）
timestamp as "timestamp!: DateTime<Utc>"
kill_time as "kill_time!: DateTime<Utc>"
created_at as "created_at!: DateTime<Utc>"
updated_at as "updated_at!: DateTime<Utc>"
```

共修复 **6 处** SQL 查询（4 处 `MessageRecord` 查询 + 2 处 `KillRecord` 查询）。

#### 2.3.2 `game/service.rs` — 同上 DateTime 修复

同样的问题出现在 `Game` 结构体的查询中。新增 `use chrono::{DateTime, Utc};` 导入，修复 **3 处** SQL 查询：
- `create_game` 中的 `SELECT ... RETURNING`
- `get_game_by_id_with_player_counts` 中的 `SELECT`
- `get_game_by_id` 中的 `SELECT`

#### 2.3.3 `main.rs` — Windows jemalloc 兼容

```rust
// 修复前：jemalloc 在非 MSVC 环境启用，但 Windows GNU 工具链也会尝试使用
#[cfg(all(not(target_env = "msvc")))]

// 修复后：所有 Windows 目标都不使用 jemalloc
#[cfg(all(not(target_env = "msvc"), not(windows)))]
```

影响 3 处：全局分配器声明、`set_allocator()` 函数实现、日志输出条件。

#### 2.3.4 `websocket/models.rs` — 同上 DateTime 修复

`save_game_state` 中的 SQL 查询也添加了 `timestamp!: DateTime<Utc>` 注解。

---

## 三、前端修改详情（Vue / TypeScript）

### 3.1 货币系统 — 类型定义层

#### 3.1.1 `types/gameStateTypes.ts`

```typescript
// ItemCategory 新增货币类型
export type ItemCategory = 'weapon' | 'armor' | 'consumable' | 'utility' | 'upgrader' | 'currency';

// Player 接口新增货币字段
export interface Player {
  // ... 原有字段
  coins: number;  // 新增
}
```

#### 3.1.2 `utils/itemConfigUtils.ts` — 货币配置解析

新增完整货币配置的类型定义和解析逻辑：

```typescript
// 新增接口
interface CurrencyProperties { value: number }
interface CurrencyConfig {
  name: string
  internalName?: string
  rarity?: string
  properties: CurrencyProperties
}

// ItemsConfig 接口新增
currencies: CurrencyConfig[]

// 解析函数中新增货币处理（与消耗品模式一致）
// 货币名称重复检测也加入了 findDuplicateItemNames
```

#### 3.1.3 `utils/itemParser.ts` — 货币物品解析

- 新增 `currencies: string[]` 到 `ParsedItems` 接口
- `hasAnyItem` 判断包含 `currencies`
- 货币名称提取加入 `allItems` 数组

#### 3.1.4 `utils/itemType.ts` — 货币分类显示

```typescript
// 分类标签映射新增
currency: '货币'

// Tag 颜色映射新增（黄色 warning 标签）
currency: 'warning'
```

#### 3.1.5 `utils/itemDisplay.ts` — 货币属性展示

新增 `case 'currency'` 分支，显示货币面值 `"面值: {value}"`。

#### 3.1.6 `utils/gameRuleParser.ts` — 货币配置校验

新增约 35 行货币验证逻辑：
- 验证 `currencies` 必须是数组
- 验证每个条目必须是对象
- 验证允许的键：`name`, `internal_name`, `rarity`, `properties`
- 验证 `name` 必须是非空字符串
- 验证 `properties.value` 必须是有限数字
- 检查 `properties` 中不允许出现未知键

### 3.2 货币系统 — UI 层

#### 3.2.1 `views/actor/components/CompactActionPanel.vue` — 演员面板

在体力值和位置之间新增货币显示行：
```html
<div class="status-value coins">货币: {{ player.coins }}</div>
```
样式：金色字体 `#e6a23c`，加粗 `font-weight: 600`。

#### 3.2.2 `views/actor/components/InventoryPanel.vue` — 背包排序

新增 `currencyItems` 数组，货币类物品排在力量类消耗品之后展示。货币物品被标记为"可使用"（使用后转化为玩家 coins 字段）。

#### 3.2.3 `views/director/components/BatchAirdropDialog.vue` — 批量空投

新增"货币"选择区域，使用 `el-input-number` 设置空投数量，与其他物品类型（武器/防具/消耗品/工具/升级器）并列。

#### 3.2.4 `views/director/components/PlayerStatusCard.vue` — 导演玩家管理

新增可排序、可编辑的"货币"列：

```html
<el-table-column label="货币" min-width="60">
  <template #default="scope">
    <el-input v-model="scope.row.coins"
      @focus="handleEditableFieldFocus(scope.row, 'coins')"
      @blur="handleCoinsBlur(scope.row, event)" />
  </template>
</el-table-column>
```

新增方法：`handleCoinsBlur`、`updatePlayerCoins`，模式与生命值/体力值编辑一致。

纯文本导出也新增了货币列。

### 3.3 导演移动玩家功能

#### 3.3.1 `stores/gameState.ts` — Store 层

新增两个方法：

```typescript
// 设置玩家货币
const setPlayerCoins = (playerId: string, coins: number) => {
  sendDirectorAction('coins', { player_id: playerId, coins })
}

// 移动玩家到指定地点
const movePlayer = (playerId: string, targetPlace: string) => {
  sendDirectorAction('move_player', { player_id: playerId, target_place: targetPlace })
}
```

均通过 WebSocket 发送导演动作，由后端 `DirectorActionScheduler` 路由到对应 handler。

#### 3.3.2 `views/director/components/PlayerStatusCard.vue` — 位置下拉选择

**初版**：位置列为只读文本显示。
```html
<el-table-column label="位置" min-width="80" prop="location" />
```

**当前**：位置列为 `el-select` 下拉选择器。

```html
<el-table-column label="位置" min-width="100" prop="location">
  <template #default="scope">
    <el-select
      :model-value="scope.row.location"
      @change="(val) => handleLocationChange(scope.row.id, val)"
      :disabled="!scope.row.is_alive">
      <el-option v-for="place in availablePlaces" :key="place" :label="place" :value="place" />
    </el-select>
  </template>
</el-table-column>
```

新增计算属性和方法：
- `availablePlaces`：从 store 获取所有地点名称列表
- `handleLocationChange(playerId, targetPlace)`：调用 `store.movePlayer()`
- 已死亡玩家的选择器被禁用（`:disabled="!scope.row.is_alive"`）

---

## 四、数据流总览

### 4.1 货币使用流程

```
玩家点击"使用"货币物品
    → player_use_action.rs: handle_use_item()
    → match ItemType::Currency(properties)
    → handle_currency_use()
        → player.coins += properties.value
        → 物品从背包移除（reinsert = false）
    → ActionResult 返回给前端
    → 前端 CompactActionPanel 显示更新后的 coins 值
```

### 4.2 死亡货币处理流程

```
玩家死亡（kill_player 被调用）
    → 提取 victim_coins，清零 player.coins
    → 检查 loot_recipient_id（击杀者）
        ├─ 有击杀者（PVP） → killer.coins += victim_coins, 日志"缴获货币"
        └─ 无击杀者（非PVP） → 货币消失, 日志"消失货币"
    → 物品按 death_item_disposition 规则处理（不变）
```

### 4.3 导演移动玩家流程

```
导演在前端选择新位置
    → PlayerStatusCard: handleLocationChange()
    → store.movePlayer(playerId, targetPlace)
    → WebSocket: sendDirectorAction('move_player', {...})
    → director_action_scheduler.rs: dispatch "move_player"
    → director_common_actions.rs: handle_move_player()
        → 验证地点存在 & 未摧毁
        → 旧地点.players.remove(playerId)
        → player.location = targetPlace
        → 新地点.players.push(playerId)
    → GameState 推送给前端
    → 前端 PlayerStatusCard 下拉框自动更新
```

---

## 五、修改文件清单

### 后端（Rust）— 9 个文件

| 文件 | 修改类型 | 改动量 |
|------|---------|--------|
| `game/game_rule_engine.rs` | 修改 | 新增 Player.coins、CurrencyProperties、CurrencyConfig、currencies 字段 |
| `game/service.rs` | 修改 | DateTime 注解修复 + chrono 导入 |
| `game/log_service.rs` | 修改 | DateTime 注解修复（6 处 SQL） |
| `main.rs` | 修改 | Windows jemalloc 条件编译（3 处） |
| `websocket/models.rs` | 修改 | DateTime 注解修复 |
| `websocket/actions/game_state_common.rs` | 修改 | kill_player 货币逻辑重构 + 新增 handle_set_player_coins + handle_move_player |
| `websocket/actions/director_action_scheduler.rs` | 修改 | 新增 coins + move_player dispatch |
| `websocket/actions/director_common_actions.rs` | 修改 | 新增 handle_set_player_coins + handle_move_player |
| `websocket/actions/player_use_action.rs` | 修改 | 新增 handle_currency_use |

### 前端（Vue / TypeScript）— 11 个文件

| 文件 | 修改类型 | 改动量 |
|------|---------|--------|
| `types/gameStateTypes.ts` | 修改 | 新增 coins + currency 类型 |
| `stores/gameState.ts` | 修改 | 新增 setPlayerCoins + movePlayer |
| `utils/itemConfigUtils.ts` | 修改 | 新增 CurrencyConfig 类型 + 解析逻辑 |
| `utils/itemParser.ts` | 修改 | 新增 currencies 解析 |
| `utils/itemType.ts` | 修改 | 新增 currency 分类标签 |
| `utils/itemDisplay.ts` | 修改 | 新增 currency 展示分支 |
| `utils/gameRuleParser.ts` | 修改 | 新增 currencies 验证（~35 行） |
| `views/actor/components/CompactActionPanel.vue` | 修改 | 新增货币显示行 |
| `views/actor/components/InventoryPanel.vue` | 修改 | 货币物品排序 + 可使用标记 |
| `views/director/components/BatchAirdropDialog.vue` | 修改 | 货币空投选项 |
| `views/director/components/PlayerStatusCard.vue` | 修改 | 位置下拉选择器 + 货币可编辑列 |

---

## 六、增量更新（2026-06-07）

> 本节记录在上述改造基础上新增的“规则一致性 + 经济安全 + 可见性边界”增强，属于后端行为修复与回归测试补齐。

### 6.1 商店购买（shop_buy）增强

涉及文件：`backend/src/websocket/actions/player_common_actions.rs`

#### 6.1.1 重复 listing 合并校验（防超卖）

原逻辑按请求逐条校验库存：
- 每条 `buy.quantity <= listing.quantity` 都可能通过
- 但同一 `listing_id` 多条请求的合计数量可能超过库存

当前逻辑：
- 先按 `listing_id` 聚合购买数量
- 再进行统一库存校验与统一扣减
- 阻断“拆单绕过库存”的路径

#### 6.1.2 总价计算溢出保护

原逻辑：
- `total_cost += listing.price * buy_qty`（`i32`）

当前逻辑：
- 对单项总价使用 `checked_mul`
- 对累计总价使用 `checked_add`
- 任何溢出直接取消交易并返回 Info 提示

同时对数量聚合与数量累计也做了边界保护，避免异常大数量导致内部状态异常。

#### 6.1.3 购买结果广播策略拆分

为满足“全体只同步商店状态；购买明细仅购买者+导演可见”的要求，`shop_buy` 返回拆分为两条结果：

1. **全体同步结果（Info）**
    - 广播对象：全体玩家 + 导演
    - 目的：触发前端商店库存刷新
    - 不作为全员可见购买明细日志

2. **定向购买明细（SystemNotice）**
    - 广播对象：购买者 + 导演
    - 内容：购买件数、花费、购买物品明细、剩余货币
    - 不对其他玩家公开明细

#### 6.1.4 扣款与库存扣减安全性

在交易提交阶段增加检查式运算：
- 玩家扣款使用 `checked_sub`
- 库存扣减使用 `checked_sub`

确保即使出现边界输入，也不会发生回绕式错误扣减。

### 6.2 货币道具使用溢出保护

涉及文件：`backend/src/websocket/actions/player_use_action.rs`

`handle_currency_use` 的货币累加从直接加法改为：
- `player.coins.checked_add(properties.value)`

行为语义：
- 溢出时拒绝使用（返回 Info）
- 且不消耗道具（保持失败原子性）

### 6.3 击杀货币转移溢出保护

涉及文件：`backend/src/websocket/actions/game_state_common.rs`

`kill_player` 中击杀者货币缴获改为检查式加法：
- `killer.coins.checked_add(victim_coins)`

当发生溢出风险时：
- 不执行回绕累加
- `transferred_coins` 保持 0
- 货币按“未成功转移”路径处理（日志体现为消失货币）

### 6.4 新增回归测试覆盖

涉及文件：`backend/tests/currency_and_movement_integration.rs`

本次新增并通过的测试覆盖点：

1. **商店交易正确性与原子性**
    - 重复 `listing_id` 合并后超库存拦截
    - 余额不足不应修改库存/背包/货币
    - 物品创建失败整笔交易回滚

2. **金额计算安全性**
    - 单项乘法溢出（`price * qty`）
    - 累计加法溢出（`total_cost`）

3. **广播可见性边界**
    - `shop_buy` 结果拆分为“全体库存同步 + 定向购买明细”
    - 校验消息类型与广播范围符合设计

4. **货币使用边界与原子性**
    - 正常边界值使用成功
    - 溢出时失败且道具不消耗

5. **死亡货币处理路径**
    - PVP / 流血致死：货币可转移
    - 缩圈 / 导演击杀：货币消失
    - 击杀转移溢出：不回绕、不中毒经济状态

6. **旧存档兼容**
    - `coins` 缺失默认值
    - `shop` 缺失默认空数组
    - `shop.quantity` 缺失默认值

### 6.5 本次增量修改文件（后端）

| 文件 | 修改类型 | 说明 |
|------|---------|------|
| `backend/src/websocket/actions/player_common_actions.rs` | 修改 | `shop_buy` 聚合校验、溢出防护、广播拆分、扣减安全 |
| `backend/src/websocket/actions/player_use_action.rs` | 修改 | 货币使用 `checked_add` + 失败原子性 |
| `backend/src/websocket/actions/game_state_common.rs` | 修改 | 击杀货币转移 `checked_add` 防回绕 |
| `backend/tests/currency_and_movement_integration.rs` | 修改 | 新增商店/货币/死亡/兼容性回归测试 |
