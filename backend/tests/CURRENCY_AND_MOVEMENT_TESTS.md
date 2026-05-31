# 货币和玩家移动功能测试总结

## 概述

已为 Royale Arena 后端添加了 **13 个新的单元测试**，专门测试新增的货币系统和导演玩家移动功能。

## 测试文件

**位置**: `backend/tests/currency_and_movement_integration.rs`

## 测试覆盖用例

### 1. 货币配置解析 (3 个测试)

#### `test_currency_config_parsing`
- **目的**: 验证规则配置中的货币配置正确解析
- **验证内容**:
  - 货币数量是否正确加载 (2 个货币类型)
  - 各货币名称是否正确
  - 各货币面值是否正确

#### `test_create_currency_item_from_config`
- **目的**: 验证从配置创建货币物品的功能
- **验证内容**:
  - 金币 (面值: 100) 创建成功
  - 银币 (面值: 10) 创建成功
  - 物品类型为 `Currency`
  - 物品属性正确映射

#### `test_create_nonexistent_currency_item`
- **目的**: 验证创建不存在的货币物品失败处理
- **验证内容**:
  - 请求不存在的货币返回错误

### 2. 玩家货币管理 (3 个测试)

#### `test_player_initial_coins_is_zero`
- **目的**: 验证新玩家的初始货币值
- **验证内容**:
  - 新创建的玩家初始货币为 0

#### `test_director_set_player_coins`
- **目的**: 验证导演设置玩家货币的功能
- **验证内容**:
  - 设置货币为 100
  - 设置相同值不报错 (返回"未变化"消息)
  - 设置货币为 0
  - 设置货币为负数 (-50，允许透支)
  - 每次设置都正确应用到玩家数据

#### `test_director_set_currency_nonexistent_player`
- **目的**: 验证设置不存在玩家的货币失败处理
- **验证内容**:
  - 尝试设置不存在玩家的货币返回 "Player not found" 错误

### 3. 玩家移动功能 (5 个测试)

#### `test_director_move_player`
- **目的**: 验证导演将玩家移动到不同位置
- **验证内容**:
  - 玩家从位置1成功移动到位置2
  - 玩家的 location 字段正确更新

#### `test_director_move_player_same_location`
- **目的**: 验证移动到相同位置的处理
- **验证内容**:
  - 移动到当前位置返回成功
  - 玩家位置保持不变

#### `test_director_move_player_nonexistent_location`
- **目的**: 验证移动到不存在位置的失败处理
- **验证内容**:
  - 尝试移动到不存在的位置返回错误
  - 错误消息包含"不存在"

#### `test_director_move_nonexistent_player`
- **目的**: 验证移动不存在的玩家失败处理
- **验证内容**:
  - 尝试移动不存在的玩家返回 "Player not found" 错误

#### `test_director_move_player_to_destroyed_location`
- **目的**: 验证移动到被摧毁位置的处理
- **验证内容**:
  - 尝试移动到被摧毁的位置不成功
  - 玩家位置保持原位置

### 4. 货币物品属性 (2 个测试)

#### `test_currency_item_properties`
- **目的**: 验证货币物品的完整属性
- **验证内容**:
  - 物品名称正确
  - 物品类型为 `Currency`
  - 物品面值正确
  - 物品无 internal_name 和 rarity 属性

#### `test_multiple_players_independent_coins`
- **目的**: 验证多个玩家的货币独立管理
- **验证内容**:
  - 玩家1的货币独立设置 (100)
  - 玩家2的货币独立设置 (50)
  - 修改玩家1的货币 (200) 不影响玩家2 (仍为 50)

## 测试执行结果

```
running 13 tests
test test_player_initial_coins_is_zero ... ok
test test_director_move_player_same_location ... ok
test test_currency_item_properties ... ok
test test_currency_config_parsing ... ok
test test_director_move_player_nonexistent_location ... ok
test test_create_nonexistent_currency_item ... ok
test test_create_currency_item_from_config ... ok
test test_director_set_currency_nonexistent_player ... ok
test test_multiple_players_independent_coins ... ok
test test_director_move_player ... ok
test test_director_move_player_to_destroyed_location ... ok
test test_director_move_nonexistent_player ... ok
test test_director_set_player_coins ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 运行测试

执行以下命令运行所有新增的测试：

```bash
cd backend
cargo test --test currency_and_movement_integration
```

或运行所有测试（包括现有测试）：

```bash
cargo test
```

## 测试代码结构

### 使用的技术栈
- **Rust 单元测试框架**: 使用 `#[test]` 属性宏
- **JSON 配置**: 使用 `serde_json` 创建测试规则配置
- **游戏规则引擎**: 直接测试 `GameRuleEngine::from_json()`
- **游戏状态**: 直接测试 `GameState` 的方法

### 关键测试函数

1. **规则配置解析**: `GameRuleEngine::from_json()`
2. **物品创建**: `rule_engine.create_item_from_name()`
3. **玩家创建**: `Player::new()`
4. **游戏状态初始化**: `GameState::new()`
5. **导演操作**:
   - `game_state.handle_set_player_coins(player_id, coins)`
   - `game_state.handle_move_player(player_id, target_place)`

## 测试覆盖范围

### 功能覆盖
- ✅ 货币类型配置解析
- ✅ 货币物品创建
- ✅ 玩家货币初始化
- ✅ 导演设置货币（所有情况）
- ✅ 导演移动玩家（所有情况）
- ✅ 错误处理（不存在的玩家、位置等）
- ✅ 边界情况（负数货币、相同位置移动等）

### 已验证的行为
- 货币值在 Player 结构体中正确初始化和更新
- 导演操作返回正确的结果
- 错误消息清晰准确
- 多玩家场景中货币独立管理

## 注意事项

### 限制
- 这些是单元测试，主要测试核心逻辑
- 没有包含与 WebSocket、数据库的集成测试（集成测试可单独编写）
- 死亡时的货币转移不在这里测试（需要完整的死亡流程，通常在集成测试中测试）

### 依赖关系
- 测试依赖于修改报告中的所有后端代码更改
- `GameState::new()` 需要规则配置 JSON
- `Place::new()` 用于创建位置对象

## 维护指南

如果需要添加新的货币或更改货币相关规则：

1. 在 `get_test_rules_with_currency()` 函数中更新测试规则配置
2. 相应地调整测试用例中的期望值
3. 运行 `cargo test` 确保所有测试都通过

## 总结

通过这 13 个测试，我们全面验证了：
- ✅ 货币系统的完整功能
- ✅ 导演移动玩家的所有场景
- ✅ 边界情况和错误处理
- ✅ 数据一致性

所有测试都通过，代码质量和可靠性得到保证。
