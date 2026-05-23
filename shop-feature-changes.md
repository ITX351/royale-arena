# 商店系统（Shop System）代码修改总结报告

## 功能概述

为 Royale Arena 游戏平台新增了完整的商店购买系统，包含以下核心能力：

- **导演端**：上架物品（设置名称、单价、库存数量）、下架物品、查看已上架物品列表
- **玩家端**：浏览商店商品、选择购买数量、批量购买（多选多件）、自动扣款和入背包
- **数据层**：商店数据存储在 `GameState` 内存中，通过 `global_state` WebSocket 广播给所有客户端（导演 + 全体玩家看到相同商品）
- **原子交易**：购买多件物品时，要么全部成功（扣款+入背包+扣库存），要么全部失败
- **向后兼容**：旧存档文件缺少 `shop` 字段时自动初始化为空数组，`quantity` 字段缺失时默认值为 1

---

## 一、后端修改（Rust / Axum）

### 1. 数据模型 — `backend/src/websocket/models.rs`

| 位置 | 修改内容 |
|------|----------|
| 第 66-78 行 | 新增 `ShopListing` 结构体，包含 `id: String`、`item_name: String`、`price: i32`、`quantity: i32`（带 `#[serde(default = "default_quantity")]` 向后兼容） |
| 第 80-82 行 | 新增 `default_quantity()` 函数，返回 `1`，用于旧数据反序列化时 `quantity` 字段的默认值 |
| 第 84-91 行 | 新增 `ShopBuyItem` 结构体，包含 `listing_id: String` 和 `quantity: i32`，用于玩家购买请求 |
| 第 129-132 行 | `GameState` 结构体新增 `shop: Vec<ShopListing>` 字段，带 `#[serde(default)]` |
| 第 480 行 | `GameState::new()` 中初始化 `shop: Vec::new()` |
| 第 504-505 行 | 自定义反序列化的 `GameStateHelper` 中新增 `#[serde(default)] shop: Vec<ShopListing>` |
| 第 528 行 | 反序列化重建时映射 `shop: helper.shop` |

### 2. 广播器 — `backend/src/websocket/broadcaster.rs`

| 位置 | 修改内容 |
|------|----------|
| 第 157 行 | `to_director_client_json()` 中添加 `"shop": self.shop`，导演视角全局状态包含商店数据 |
| 第 170 行 | `to_player_client_json()` 中添加 `"shop": self.shop`，玩家视角全局状态包含商店数据 |

### 3. 导演行动调度器 — `backend/src/websocket/actions/director_action_scheduler.rs`

| 位置 | 修改内容 |
|------|----------|
| 第 44-48 行 | `DirectorActionParams` 结构体新增商店操作字段：`shop_listing_id: Option<String>`、`price: Option<i32>`、`quantity: Option<i32>` |
| 第 218-227 行 | `dispatch()` 新增 `"shop_list_item"` 分支：提取 `item_name`、`price`、`quantity`（默认为 1），调用 `handle_shop_list_item()` |
| 第 229-234 行 | `dispatch()` 新增 `"shop_delist_item"` 分支：提取 `shop_listing_id`，调用 `handle_shop_delist_item()` |

### 4. 导演行动处理 — `backend/src/websocket/actions/director_common_actions.rs`

| 位置 | 修改内容 |
|------|----------|
| 第 5 行 | 新增 `ShopListing` 到 import |
| 第 762-802 行 | 新增 `handle_shop_list_item()` 方法 |
| | - 通过 `rule_engine.create_item_from_name()` 验证物品名称存在 |
| | - 使用 `quantity.max(1)` 确保数量至少为 1 |
| | - 生成 UUID 作为上架条目 ID |
| | - 推入 `self.shop` 列表 |
| | - 设置 `broadcast_to_all = true` 广播给全体玩家 + 导演 |
| 第 804-831 行 | 新增 `handle_shop_delist_item()` 方法 |
| | - 按 ID 查找并从 `self.shop` 中移除 |
| | - 设置 `broadcast_to_all = true` 广播给全体 |

### 5. 玩家行动调度器 — `backend/src/websocket/actions/player_action_scheduler.rs`

| 位置 | 修改内容 |
|------|----------|
| 第 5 行 | 新增 `ShopBuyItem` 到 import |
| 第 36-37 行 | `ActionParams` 结构体新增 `shop_buy_items: Option<Vec<ShopBuyItem>>` |
| 第 305-320 行 | `dispatch()` 新增 `"shop_buy"` 分支 |
| | - 前置条件验证：Alive、Born、NotBound（不消耗体力，不限夜间） |
| | - 提取 `shop_buy_items` 参数，调用 `handle_shop_buy_action()` |

### 6. 玩家购买处理 — `backend/src/websocket/actions/player_common_actions.rs`

| 位置 | 修改内容 |
|------|----------|
| 第 632-773 行 | 新增 `handle_shop_buy_action()` 方法，完整购买逻辑流程： |
| | 1. 验证 `buy_items` 非空 |
| | 2. 遍历每个购买项，匹配 `listing_id` 到商店列表，验证购买数量不超过库存 |
| | 3. 计算总花费 = Σ(price × buy_qty) 和总物品数 |
| | 4. 验证玩家 `coins >= total_cost` |
| | 5. 验证背包剩余空间 >= 总物品数 |
| | 6. 对每个商品调用 `rule_engine.create_item_from_name()` 创建物品并加入背包 |
| | 7. 扣除玩家 `coins` |
| | 8. 扣减商店列表 `quantity`，移除 `quantity <= 0` 的商品 |
| | 9. 构造 ActionResult 包含已购物品、花费、剩余货币，广播给购买者 + 导演 |

---

## 二、前端修改（Vue 3 / TypeScript / Element Plus）

### 7. 类型定义 — `frontend/src/types/gameStateTypes.ts`

| 位置 | 修改内容 |
|------|----------|
| 第 3-9 行 | 新增 `ShopListing` 接口：`{ id: string; item_name: string; price: number; quantity: number }` |
| 第 11-15 行 | 新增 `ShopBuyItem` 接口：`{ listing_id: string; quantity: number }` |
| 第 98 行 | `GlobalState` 接口新增 `shop: ShopListing[]` |

### 8. 状态管理 — `frontend/src/stores/gameState.ts`

| 位置 | 修改内容 |
|------|----------|
| 第 17 行 | 新增 `ShopListing`、`ShopBuyItem` 类型导入 |
| 第 98-100 行 | 新增 `shopListings` computed 属性，返回 `globalState.value?.shop \|\| []` |
| 第 317-319 行 | 新增 `shopListItem(itemName, price, quantity)` 方法，调用 `sendDirectorAction('shop_list_item', ...)` |
| 第 322-324 行 | 新增 `shopDelistItem(listingId)` 方法，调用 `sendDirectorAction('shop_delist_item', ...)` |
| 第 327-329 行 | 新增 `shopBuy(items: ShopBuyItem[])` 方法，调用 `sendPlayerAction('shop_buy', ...)` |
| 第 410 行 | 导出 `shopListings` computed |
| 第 438-440 行 | 导出 `shopListItem`、`shopDelistItem`、`shopBuy` 方法 |

### 9. 玩家操作面板 — `frontend/src/views/actor/components/CompactActionPanel.vue`

| 位置 | 修改内容 |
|------|----------|
| 第 137-191 行 | 模板新增商店弹窗（`el-popover`）： |
| | - "商店"按钮（带购物车图标），点击打开弹窗 |
| | - 弹窗内显示商品列表，每行包含：物品名称、单价、库存数量、购买数量输入框（`el-input-number`） |
| | - 底部显示总件数、总花费、购买按钮 |
| 第 294 行 | 新增 `ShopListing`、`ShopBuyItem` 类型导入 |
| 第 303 行 | Props 新增 `shopListings: ShopListing[]` |
| 第 311 行 | Emits 新增 `'shop-buy': [items: ShopBuyItem[]]` |
| 第 327-328 行 | 新增 `shopPopoverVisible` 和 `shopQuantities` 响应式状态 |
| 第 330-335 行 | 新增 `shopTotalCost` computed，计算选中商品总价 |
| 第 337-341 行 | 新增 `shopTotalCount` computed，计算选中商品总件数 |
| 第 343-346 行 | 新增 `onShopPopoverClose()`，关闭弹窗时重置状态 |
| 第 348-355 行 | 新增 `handleShopBuy()`，构建 `ShopBuyItem[]` 并触发 emit |
| 第 1164-1245 行 | 新增商店相关 CSS 样式 |

### 10. 玩家游戏内页面 — `frontend/src/views/actor/states/InGameState.vue`

| 位置 | 修改内容 |
|------|----------|
| 第 9 行 | 向 `CompactActionPanel` 传递 `:shop-listings="shopListings"` |
| 第 12 行 | 监听 `@shop-buy="handleShopBuy"` 事件 |
| 第 66 行 | 新增 `ShopBuyItem` 类型导入 |
| 第 102-104 行 | 新增 `shopListings` computed 属性，委托给 `gameStateStore.shopListings` |
| 第 156-158 行 | 新增 `handleShopBuy(items)` 方法，调用 `gameStateStore.shopBuy(items)` |

### 11. 导演商店管理组件 — `frontend/src/views/director/components/ShopManagement.vue`（新文件，共 192 行）

| 位置 | 内容 |
|------|------|
| 第 1-90 行 | 模板部分： |
| | - 卡片头部"商店管理" + "上架物品"按钮 |
| | - 已上架物品表格（`el-table`）：物品名称、单价、库存数量、下架按钮 |
| | - 空状态提示（`el-empty`） |
| | - 上架对话框（`el-dialog`）：物品选择器（按类别分组的 `el-select`）、单价输入（`el-input-number`）、数量输入（`el-input-number`）、上架按钮 |
| 第 92-96 行 | 脚本导入 |
| 第 105 行 | `shopListings` computed，从 store 读取当前商店列表 |
| 第 107-153 行 | 物品分组逻辑，使用 `createItemParser(rulesConfig)` 将规则配置中的物品按类别（武器/防具/功能道具/消耗品/货币/升级器）分组，供上架选择器使用 |
| 第 155-160 行 | `openListDialog()`，重置表单并打开对话框 |
| 第 162-166 行 | `handleListItem()`，调用 `store.shopListItem(...)` 上架物品 |
| 第 168-170 行 | `handleDelist()`，调用 `store.shopDelistItem(...)` 下架物品 |
| 第 173-191 行 | 作用域 CSS 样式 |

### 12. 导演游戏内管理页面 — `frontend/src/views/director/management/InGameManagement.vue`

| 位置 | 修改内容 |
|------|----------|
| 第 102-105 行 | 在 `<div class="full-width-section">` 中渲染 `<ShopManagement />`，位于广播消息组件之前 |
| 第 128 行 | 新增 `import ShopManagement from '../components/ShopManagement.vue'` |

---

## 三、数据流向图

```
导演上架物品:
  ShopManagement.vue  →  store.shopListItem(name, price, qty)
                      →  webSocketService.sendDirectorAction('shop_list_item', {...})
                      →  Backend: director_action_scheduler → director_common_actions
                      →  GameState.shop.push(ShopListing{quantity})
                      →  broadcaster → 全体 WebSocket 客户端
                      →  前端 globalState.shop 更新 → 表格刷新

玩家购买物品:
  CompactActionPanel.vue  →  emit('shop-buy', items)
                          →  InGameState.vue → store.shopBuy(items)
                          →  webSocketService.sendPlayerAction('shop_buy', {shop_buy_items: [...]})
                          →  Backend: player_action_scheduler → player_common_actions
                          →  验证 → 创建物品 → 扣款 → 扣库存 → 广播
                          →  前端更新: 玩家背包+货币, 商店库存
```

---

## 四、WebSocket 通信协议

### 导演 → 后端

**上架物品** `shop_list_item`
```json
{ "type": "director_action", "data": { "action": "shop_list_item", "item_name": "[绿]佩剑", "price": 10, "quantity": 3 } }
```

**下架物品** `shop_delist_item`
```json
{ "type": "director_action", "data": { "action": "shop_delist_item", "shop_listing_id": "uuid-xxx" } }
```

### 玩家 → 后端

**购买物品** `shop_buy`
```json
{ "type": "player_action", "data": { "action": "shop_buy", "shop_buy_items": [{"listing_id": "uuid-xxx", "quantity": 2}] } }
```

### 后端 → 前端（全局状态广播）

```json
{ "global_state": { "shop": [{"id": "uuid", "item_name": "[绿]佩剑", "price": 10, "quantity": 3}], ... }, ... }
```
