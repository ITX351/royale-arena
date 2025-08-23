# 游戏规则存储设计文档

## 概述

本文档描述了 Royale Arena 游戏规则的存储设计方案。为了提高灵活性和可扩展性，我们采用 JSON 格式存储游戏规则，避免频繁修改数据库表结构。

## 设计原则

1. **极致灵活性**: 规则结构可以随时扩展，无需修改数据库表结构
2. **简化数据库**: 数据库表结构保持简单，只存储必要的元数据
3. **快速迭代**: 添加新功能时只需修改代码，无需数据库迁移
4. **向后兼容**: 支持默认值机制，确保旧数据与新规则兼容

## 数据库表结构

### rule_templates 表

```sql
CREATE TABLE rule_templates (
    id VARCHAR(36) PRIMARY KEY COMMENT '模版唯一标识符(UUID)',
    template_name VARCHAR(100) NOT NULL COMMENT '模版名称',
    description TEXT COMMENT '模版描述',
    is_active BOOLEAN NOT NULL DEFAULT TRUE COMMENT '模版是否激活',
    rules_config JSON NOT NULL COMMENT '完整的游戏规则配置(JSON格式)',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '模版创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '模版更新时间',
    
    INDEX idx_rule_templates_name (template_name),
    INDEX idx_rule_templates_active (is_active)
) COMMENT '游戏规则模版表';
```

## JSON 规则结构

关于`rules_config`字段的详细结构，请参考[游戏规则配置说明](../api/game-rules-config.md)文档。

## 代码实现建议

### Rust 数据结构定义

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTemplate {
    pub id: String,
    pub template_name: String,
    pub description: String,
    pub is_active: bool,
    pub rules_config: serde_json::Value,  // 使用serde_json::Value存储规则配置
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
```

### 默认值处理

```rust
impl RuleTemplate {
    // 从JSON解析规则，缺失字段使用默认值
    pub fn parse_rules_with_defaults(rules_json: &str) -> Result<serde_json::Value, serde_json::Error> {
        let parsed: serde_json::Value = serde_json::from_str(rules_json)?;
        // 在实际实现中，这里可以添加默认值处理逻辑
        Ok(parsed)
    }
}
```

## 扩展性说明

### 添加新规则字段

当需要添加新的规则字段时：

1. 在[游戏规则配置说明](../api/game-rules-config.md)文档中更新规则结构
2. 在代码中添加对新字段的处理逻辑
3. 更新默认值处理函数（如需要）
4. 无需修改数据库表结构

## 优势总结

1. **无需数据库迁移**: 添加新规则字段时无需修改数据库表结构
2. **灵活扩展**: 可以轻松添加任意类型的规则字段
3. **简化维护**: 数据库结构简单，易于维护
4. **快速开发**: 新功能开发时只需关注代码实现，无需考虑数据库变更
5. **向后兼容**: 通过默认值机制确保旧数据与新规则兼容