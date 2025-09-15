# 游戏实例表结构迁移设计

## 1. 概述

本迁移任务旨在将游戏实例表（games）从通过外键关联规则模板改为直接存储具体的JSON规则配置。此变更将提高游戏实例的独立性，避免因规则模板变更或删除而影响已创建的游戏实例。

### 核心变更
- 删除 `games.rule_template_id` 外键字段
- 添加 `games.rules_config` JSON字段
- 修改后端业务逻辑以适配新表结构
- 调整前端界面逻辑

## 2. 数据库迁移

### 2.1 迁移文件设计

#### Up Migration (`20250826_remove_rule_template_foreign_key.up.sql`)

```sql
-- 添加新的规则配置字段
ALTER TABLE games 
ADD COLUMN rules_config JSON NULL 
COMMENT '游戏规则配置(JSON格式)' 
AFTER max_players;

-- 迁移现有数据：将关联的规则模板配置复制到游戏表中
UPDATE games g 
INNER JOIN rule_templates rt ON g.rule_template_id = rt.id 
SET g.rules_config = rt.rules_config;

-- 删除外键约束
ALTER TABLE games 
DROP FOREIGN KEY games_ibfk_1;

-- 删除rule_template_id字段
ALTER TABLE games 
DROP COLUMN rule_template_id;
```

#### Down Migration (`20250826_remove_rule_template_foreign_key.down.sql`)

```sql
-- 重新添加rule_template_id字段
ALTER TABLE games 
ADD COLUMN rule_template_id VARCHAR(36) NULL 
COMMENT '关联的规则模板ID' 
AFTER max_players;

-- 删除rules_config字段
ALTER TABLE games 
DROP COLUMN rules_config;

-- 重新创建外键约束
ALTER TABLE games 
ADD CONSTRAINT games_ibfk_1 
FOREIGN KEY (rule_template_id) REFERENCES rule_templates(id) 
ON DELETE SET NULL;
```

### 2.2 数据迁移策略

1. **数据保护**: 在删除外键字段前，先将关联的规则配置复制到新字段
2. **兼容性**: 新增字段允许NULL值，确保迁移过程平稳
3. **可回滚**: 提供完整的回滚脚本，但数据无法完全恢复（规则配置与模板的关联关系会丢失）

## 3. 后端架构调整

### 3.1 数据模型变更

#### Game模型更新
```rust
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub director_password: String,
    pub max_players: i32,
    pub status: GameStatus,
    pub rules_config: Option<serde_json::Value>, // 新增
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### GameWithRules模型调整
```rust
#[derive(Debug, Serialize)]
pub struct GameWithRules {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: GameStatus,
    pub player_count: i32,
    pub max_players: i32,
    pub created_at: DateTime<Utc>,
    pub director_password: Option<String>,
    // 修改: 直接包含规则配置而非模板信息
    pub rules_config: Option<serde_json::Value>,
}
```

#### CreateGameRequest保持不变
- 继续接受 `rule_template_id` 参数
- 在业务逻辑层转换为具体的规则配置

#### UpdateGameRequest调整
```rust
#[derive(Debug, Deserialize)]
pub struct UpdateGameRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub director_password: Option<String>,
    pub max_players: Option<i32>,
    pub rules_config: Option<serde_json::Value>, // 新增（内部使用）
}
```

### 3.2 业务逻辑调整

#### 创建游戏逻辑
```rust
// service.rs - create_game方法
pub async fn create_game(&self, request: CreateGameRequest) -> Result<Game, GameError> {
    // 1. 验证请求参数
    request.validate().map_err(GameError::ValidationError)?;
    
    // 2. 检查游戏名称唯一性
    // ... 现有逻辑保持不变
    
    // 3. 处理规则配置
    let rules_config = if let Some(template_id) = &request.rule_template_id {
        // 从规则模板获取配置
        let template = self.get_rule_template(template_id).await?;
        Some(template.rules_config)
    } else {
        None
    };
    
    // 4. 插入游戏记录
    sqlx::query!(
        r#"
        INSERT INTO games (id, name, description, director_password, max_players, status, rules_config)
        VALUES (?, ?, ?, ?, ?, 'waiting', ?)
        "#,
        game_id,
        request.name,
        request.description,
        request.director_password,
        request.max_players,
        rules_config
    )
    .execute(&self.pool)
    .await?;
    
    // 5. 返回创建的游戏
    self.get_game_by_id(&game_id).await
}
```

#### 获取游戏详情逻辑
```rust
// service.rs - get_game_with_rules方法
pub async fn get_game_with_rules(&self, game_id: &str, include_director_password: bool) -> Result<GameWithRules, GameError> {
    // 直接查询游戏表，无需JOIN规则模板表
    let game_info = sqlx::query!(
        r#"
        SELECT id, name, description, status as "status: GameStatus", 
               max_players, created_at, director_password, rules_config
        FROM games
        WHERE id = ?
        "#,
        game_id
    )
    .fetch_optional(&self.pool)
    .await?;

    let game_info = game_info.ok_or(GameError::GameNotFound)?;
    
    // 获取玩家数量
    let player_count = self.get_player_count(game_id).await?;

    // 构建响应
    Ok(GameWithRules {
        id: game_info.id,
        name: game_info.name,
        description: game_info.description,
        status: game_info.status,
        player_count,
        max_players: game_info.max_players,
        created_at: game_info.created_at,
        director_password: if include_director_password { 
            Some(game_info.director_password) 
        } else { 
            None 
        },
        rules_config: game_info.rules_config,
    })
}
```

#### 更新游戏逻辑
```rust
// service.rs - update_game方法（简化版）
pub async fn update_game(&self, game_id: &str, request: UpdateGameRequest) -> Result<Game, GameError> {
    // 移除rule_template_id相关的更新逻辑
    // 只更新基本信息：name, description, director_password, max_players
    
    if let Some(name) = &request.name {
        sqlx::query!("UPDATE games SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?", name, game_id)
            .execute(&self.pool).await?;
    }
    
    // ... 其他字段更新逻辑
    
    self.get_game_by_id(game_id).await
}
```

### 3.3 辅助方法

```rust
// service.rs - 新增辅助方法
async fn get_rule_template(&self, template_id: &str) -> Result<RuleTemplate, GameError> {
    let template = sqlx::query!(
        "SELECT id, template_name, description, rules_config FROM rule_templates WHERE id = ? AND is_active = true",
        template_id
    )
    .fetch_optional(&self.pool)
    .await?;
    
    template.ok_or(GameError::RuleTemplateNotFound)
        .map(|t| RuleTemplate {
            id: t.id,
            template_name: t.template_name,
            description: t.description,
            rules_config: t.rules_config,
        })
}
```

## 4. 前端界面调整

### 4.1 创建游戏界面
- **保持不变**: 继续提供规则模板选择功能
- **用户体验**: 用户仍可选择模板创建游戏，后端自动将模板配置转换为游戏实例的规则配置

### 4.2 编辑游戏界面
- **移除功能**: 删除规则模板选择相关的UI组件和逻辑
- **界面简化**: 编辑表单只包含基本信息字段

#### AdminGamesPage.vue调整
```vue
<!-- 编辑游戏表单：移除规则模板选择 -->
<el-form-item v-if="!editingGame" label="规则模版" prop="rule_template_id">
  <el-select 
    v-model="gameForm.rule_template_id" 
    placeholder="请选择规则模版（可选）"
    clearable
  >
    <el-option 
      v-for="template in ruleTemplates"
      :key="template.id"
      :label="template.template_name"
      :value="template.id"
    />
  </el-select>
</el-form-item>
```

### 4.3 游戏详情界面
- **显示调整**: 直接显示游戏的规则配置，而非模板信息
- **数据结构**: 适配新的API响应格式

## 5. API接口变更

### 5.1 响应格式变更

#### 游戏详情API响应
```json
// 原格式
{
  "id": "game-id",
  "name": "游戏名称",
  // ... 其他字段
  "rule_template": {
    "id": "template-id",
    "template_name": "模板名称",
    "description": "模板描述",
    "rules_config": { /* 规则配置 */ }
  }
}

// 新格式
{
  "id": "game-id",
  "name": "游戏名称",
  // ... 其他字段
  "rules_config": { /* 直接的规则配置 */ }
}
```

#### 游戏列表API响应
- **保持兼容**: 列表接口的响应格式保持不变
- **性能优化**: 无需JOIN查询规则模板表

### 5.2 请求格式
- **创建游戏**: 请求格式保持不变，继续接受`rule_template_id`
- **更新游戏**: 移除`rule_template_id`字段

## 6. 数据一致性与迁移风险

### 6.1 风险评估
1. **数据丢失风险**: 
   - 迁移后无法追溯游戏使用的具体模板
   - 回滚操作无法恢复模板关联关系

2. **业务影响**:
   - 已创建游戏的规则配置将固化，不受模板后续变更影响
   - 编辑游戏时无法重新选择规则模板

### 6.2 迁移步骤
1. **备份数据**: 执行迁移前完整备份数据库
2. **应用迁移**: 执行Up脚本，完成表结构调整
3. **部署代码**: 更新后端服务代码
4. **验证功能**: 测试创建、编辑、查询游戏功能
5. **前端部署**: 更新前端代码

### 6.3 回滚计划
- **表结构回滚**: 使用Down脚本恢复原表结构
- **代码回滚**: 恢复原版本代码
- **数据修复**: 手动修复rule_template_id关联关系（如需要）

## 7. 测试策略

### 7.1 数据库迁移测试
- 在测试环境验证迁移脚本的正确性
- 测试数据迁移的完整性
- 验证回滚脚本的有效性

### 7.2 功能测试
- 创建游戏功能（带模板和不带模板）
- 编辑游戏功能（验证无模板选择选项）
- 游戏详情查询功能
- API兼容性测试

### 7.3 性能测试
- 对比迁移前后的查询性能
- 验证移除JOIN查询的性能提升效果

## 8. 实施计划

### 阶段1: 数据库迁移 (1-2小时)
1. 创建迁移文件
2. 在开发环境测试迁移
3. 执行生产环境迁移

### 阶段2: 后端代码更新 (2-3小时)
1. 更新数据模型
2. 修改业务逻辑
3. 调整API接口
4. 编写单元测试

### 阶段3: 前端代码更新 (1-2小时)
1. 调整游戏编辑界面
2. 更新API调用逻辑
3. 测试用户界面

### 阶段4: 集成测试 (1小时)
1. 端到端功能测试
2. 性能验证
3. 用户验收测试

## 9. 监控与维护

### 9.1 迁移后监控
- 监控游戏创建和编辑功能的错误率
- 检查API响应时间改善情况
- 验证数据完整性

### 9.2 后续维护
- 定期检查游戏规则配置的一致性
- 监控存储空间使用情况（JSON配置可能增加存储需求）
- 考虑建立规则配置的版本管理机制
