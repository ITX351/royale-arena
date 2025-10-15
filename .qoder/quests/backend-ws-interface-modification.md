# Royale Arena WebSocket接口修改设计文档

## 1. 概述

本文档描述了对Royale Arena系统WebSocket接口的修改需求，包括后端移除旧的`give`功能并增加新的玩家物品管理接口，以及前端导演控制台界面的布局调整和功能细化。

### 1.1 修改目标

- **后端修改**：
  - 移除旧的`give`功能
  - 增加"向玩家添加物品"和"从玩家移除物品"两个WebSocket管理接口
  - 确保导演修改玩家状态时向玩家发出通知

- **前端修改**：
  - 调整导演页布局，PlaceStatusCard和PlayerStatusCard横跨整个屏幕
  - 细化玩家状态管理，允许查看和修改玩家详细状态
  - 增加状态复制功能

### 1.2 设计原则

- 所有修改均为破坏性修改，不考虑向后兼容
- 实现最小功能集，拒绝过度设计
- 所有输出使用简体中文

## 2. 后端架构修改

### 2.1 接口变更

#### 2.1.1 移除的接口

| 接口名称 | 描述 |
|---------|------|
| `give` | 旧的道具增减接口 |

#### 2.1.2 新增的接口

| 接口名称 | 参数 | 描述 |
|---------|------|------|
| `add_player_item` | `player_id`, `item` | 向玩家添加物品 |
| `remove_player_item` | `player_id`, `item_name` | 从玩家移除物品 |

### 2.2 核心组件修改

#### 2.2.1 DirectorActionScheduler修改

```rust
// 新增参数字段
pub struct DirectorActionParams {
    // ... 现有字段 ...
    
    /// 玩家物品操作
    pub item_name: Option<String>,
}
```

```rust
// 修改调度逻辑
match action_type {
    // ... 现有匹配项 ...
    
    "add_player_item" => {
        let player_id = action_params.player_id
            .ok_or_else(|| "Missing player_id parameter".to_string())?;
        let item = action_params.item
            .ok_or_else(|| "Missing item parameter".to_string())?;
        game_state.handle_add_player_item(&player_id, item)
    }
    
    "remove_player_item" => {
        let player_id = action_params.player_id
            .ok_or_else(|| "Missing player_id parameter".to_string())?;
        let item_name = action_params.item_name
            .ok_or_else(|| "Missing item_name parameter".to_string())?;
        game_state.handle_remove_player_item(&player_id, &item_name)
    }
    
    // 移除"give"匹配项
}
```

#### 2.2.2 GameState导演行为处理修改

```rust
impl GameState {
    // 新增方法
    /// 向玩家添加物品
    pub fn handle_add_player_item(&mut self, player_id: &str, item: Item) -> Result<ActionResult, String> {
        // 获取玩家
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 添加物品到玩家背包
        player.inventory.push(item.clone());
        
        // 构造响应数据
        let data = serde_json::json!({
            "player_id": player_id,
            "item": item,
            "action": "add"
        });
        
        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string()], 
            format!("导演向玩家 {} 添加了物品 {}", player.name, item.name)
        );
        
        Ok(action_result)
    }
    
    /// 从玩家移除物品
    pub fn handle_remove_player_item(&mut self, player_id: &str, item_name: &str) -> Result<ActionResult, String> {
        // 获取玩家
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 查找并移除物品
        let item_pos = player.inventory.iter().position(|i| i.name == item_name)
            .ok_or("Item not found in player's inventory".to_string())?;
        let removed_item = player.inventory.remove(item_pos);
        
        // 构造响应数据
        let data = serde_json::json!({
            "player_id": player_id,
            "item": removed_item,
            "action": "remove"
        });
        
        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string()], 
            format!("导演从玩家 {} 移除了物品 {}", player.name, item_name)
        );
        
        Ok(action_result)
    }
    
    // 修改现有方法以确保广播给玩家
    /// 调整生命值
    pub fn handle_life(&mut self, player_id: &str, life_change: i64) -> Result<ActionResult, String> {
        // ... 现有实现 ...
        
        // 确保在创建动作结果时广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string()], 
            format!("导演调整玩家 {} 生命值 {}", player.name, if life_change > 0 { format!("+{}", life_change) } else { life_change.to_string() })
        );
        
        Ok(action_result)
    }
    
    /// 调整体力值
    pub fn handle_strength(&mut self, player_id: &str, strength_change: i64) -> Result<ActionResult, String> {
        // ... 现有实现 ...
        
        // 确保在创建动作结果时广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string()], 
            format!("导演调整玩家 {} 体力值 {}", player.name, if strength_change > 0 { format!("+{}", strength_change) } else { strength_change.to_string() })
        );
        
        Ok(action_result)
    }
}
```

## 3. 前端架构修改

### 3.1 页面布局调整

#### 3.1.1 InGameManagement.vue修改

```vue
<template>
  <div class="in-game-management">
    <el-card class="management-card">
      <template #header>
        <div class="card-header">
          <h3>游戏中管理</h3>
        </div>
      </template>
      
      <div class="management-content">
        <!-- 横向排列的控制面板 -->
        <div class="control-section">
          <!-- 第一行：天气控制和夜晚时间设置并排 -->
          <div class="horizontal-controls">
            <!-- 天气控制 -->
            <el-card class="control-card" shadow="hover">
              <!-- ... 现有实现 ... -->
            </el-card>
            
            <!-- 夜晚时间设置 -->
            <el-card class="control-card" shadow="hover">
              <!-- ... 现有实现 ... -->
            </el-card>
          </div>
          
          <!-- 第二行：下一轮缩圈位置独占一行 -->
          <div class="full-width-control">
            <el-card class="control-card full-width-card" shadow="hover">
              <!-- ... 现有实现 ... -->
            </el-card>
          </div>
        </div>
        
        <!-- 地点状态管理和玩家状态管理卡片 - 修改为横跨整个屏幕 -->
        <div class="full-width-section">
          <PlaceStatusCard 
            :places="placeList" 
            @place-status-change="handlePlaceStatusChange"
          />
        </div>
        
        <div class="full-width-section">
          <PlayerStatusCard 
            :players="playerList" 
            @player-binding-change="handlePlayerBindingChange"
          />
        </div>
        
        <!-- 空投设置面板 -->
        <AirdropPanel 
          :game-id="game.id"
          @airdrop-accepted="handleAirdropAccepted"
        />
        
        <!-- 广播消息面板 -->
        <BroadcastMessage 
          :game-id="game.id"
          :players="playerList"
          @message-sent="handleMessageSent"
        />
      </div>
    </el-card>
  </div>
</template>

<style scoped>
/* ... 现有样式 ... */

/* 新增样式 - 全宽部分 */
.full-width-section {
  width: 100%;
}

/* 调整PlaceStatusCard和PlayerStatusCard样式 */
.full-width-section :deep(.el-card) {
  width: 100%;
}
</style>
```

### 3.2 玩家状态管理细化

#### 3.2.1 PlayerStatusCard.vue功能增强

需要在PlayerStatusCard组件中实现以下功能：
1. 允许导演查看每个玩家的全部状态
2. 允许导演修改玩家生命值、体力值
3. 允许导演向单个玩家添加物品或删除玩家物品
4. 增加状态复制按钮

### 3.3 地点状态管理细化

#### 3.3.1 PlaceStatusCard.vue功能增强

需要在PlaceStatusCard组件中实现以下功能：
1. 增加状态复制按钮，用于复制当前状态

### 3.4 WebSocket Store修改

#### 3.4.1 gameState.ts修改

```typescript
// 新增方法
const addPlayerItem = (playerId: string, item: any) => {
  sendDirectorAction('add_player_item', { player_id: playerId, item })
}

const removePlayerItem = (playerId: string, itemName: string) => {
  sendDirectorAction('remove_player_item', { player_id: playerId, item_name: itemName })
}
```

## 4. 数据模型

### 4.1 DirectorActionParams模型扩展

| 字段名 | 类型 | 必需 | 描述 |
|-------|------|------|------|
| `item_name` | `Option<String>` | 否 | 物品名称，用于移除玩家物品 |

### 4.2 ActionResult模型

确保所有导演修改玩家状态的操作都生成包含玩家ID的广播列表，以便向玩家发送通知。

## 5. 业务逻辑层

### 5.1 玩家物品管理逻辑

1. **添加物品**：
   - 验证玩家存在
   - 将物品添加到玩家背包
   - 生成包含玩家ID的广播列表
   - 记录操作日志

2. **移除物品**：
   - 验证玩家存在
   - 验证物品在玩家背包中存在
   - 从玩家背包中移除物品
   - 生成包含玩家ID的广播列表
   - 记录操作日志

### 5.2 状态通知逻辑

所有导演修改玩家状态的操作（生命值、体力值、物品）都必须：
1. 在ActionResult中包含被修改玩家的ID
2. 生成明确说明是导演修改的日志消息
3. 将消息广播给被修改的玩家和所有导演

## 6. 测试策略

不编写任何测试。