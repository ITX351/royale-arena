# Royale Arena 数据库变更总结

## 概述

本文档总结了 Royale Arena 项目数据库结构的最新变更，包括删除的表、新增的表以及规则设计格式的修改。

## 删除的表

### 1. game_stats (游戏统计数据表)
- **原因**: 数据冗余，可以通过查询其他表动态计算获得
- **影响**: 移除了相关的API接口和数据模型文档
- **替代方案**: 通过动态查询计算统计数据

### 2. votes (投票记录表)
- **原因**: 功能价值较低，决定替换为更有意义的击杀记录
- **影响**: 移除了投票相关的API接口和数据模型文档

## 新增的表

### 1. kill_records (击杀记录表)
- **目的**: 记录玩家之间的击杀事件和非玩家击杀事件
- **字段结构**:
  - `id`: VARCHAR(36) PRIMARY KEY - 击杀记录唯一标识符
  - `game_id`: VARCHAR(36) NOT NULL - 所属游戏ID
  - `killer_id`: VARCHAR(36) NULL - 击杀者ID（可为空，表示非玩家击杀）
  - `victim_id`: VARCHAR(36) NOT NULL - 被击杀者ID
  - `kill_time`: TIMESTAMP DEFAULT CURRENT_TIMESTAMP - 击杀时间
  - `cause`: VARCHAR(50) NOT NULL - 击杀原因（如：武器、缩圈等）
  - `weapon`: VARCHAR(50) NULL - 使用的武器/方式
  - `location`: VARCHAR(100) NULL - 击杀地点
- **索引**:
  - PRIMARY KEY (id)
  - INDEX idx_kill_records_game_id (game_id)
  - INDEX idx_kill_records_killer_id (killer_id)
  - INDEX idx_kill_records_victim_id (victim_id)
  - INDEX idx_kill_records_kill_time (kill_time)
- **外键约束**:
  - FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
  - FOREIGN KEY (killer_id) REFERENCES actors(id) ON DELETE CASCADE
  - FOREIGN KEY (victim_id) REFERENCES actors(id) ON DELETE CASCADE

## 修改的表

### 1. rule_templates (游戏规则模版表)
- **修改内容**: 重构表结构，采用JSON格式存储规则配置
- **字段变更**:
  - 移除了具体的规则字段（如day_duration, max_life等）
  - 新增`rules_config` JSON字段存储完整规则配置
  - 新增`is_active` BOOLEAN字段标识模板是否激活
- **字段结构**:
  - `id`: VARCHAR(36) PRIMARY KEY - 模版唯一标识符
  - `template_name`: VARCHAR(100) NOT NULL - 模版名称
  - `description`: TEXT - 模版描述
  - `is_active`: BOOLEAN NOT NULL DEFAULT TRUE - 模版是否激活
  - `rules_config`: JSON NOT NULL - 完整的游戏规则配置
  - `created_at`: TIMESTAMP DEFAULT CURRENT_TIMESTAMP - 模版创建时间
  - `updated_at`: TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP - 模版更新时间
- **索引**:
  - PRIMARY KEY (id)
  - INDEX idx_rule_templates_name (template_name)
  - INDEX idx_rule_templates_active (is_active)

## 规则设计格式

### JSON规则配置结构
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

### 扩展性设计
- 通过JSON格式存储规则，无需修改数据库表结构即可添加新规则字段
- 支持默认值机制，确保旧数据与新规则兼容
- 详细规则配置请参考[游戏规则配置说明](./api/game-rules-config.md)文档

## 索引策略优化

### 统一索引声明方式
- 所有索引都在表定义中直接声明，而非后续追加
- 提高了表结构定义的清晰度和一致性
- 避免了索引定义分散的问题

## 文档更新

### 已更新的文档
1. `backend/sql/schema.sql` - 数据库结构定义
2. `backend/sql/test_data.sql` - 测试数据插入脚本
3. `docs/api/data-models.md` - 数据模型定义
4. `docs/api/rest/director.md` - 导演接口文档
5. `docs/backend/api-task-list.md` - API任务列表

### 已删除的文档内容
1. 移除了与`game_stats`表相关的所有API接口和数据模型
2. 移除了与`votes`表相关的所有API接口和数据模型
3. 更新了相关接口编号以保持连续性

## 下一步工作

### 数据库相关
1. 更新后端代码以适配新的数据库结构
2. 实现`kill_records`表的数据访问逻辑
3. 实现`rule_templates`表的JSON规则配置处理逻辑
4. 移除与已删除表相关的代码

### API接口实现
1. 实现新的"获取击杀记录"接口
2. 更新"获取游戏规则"接口以适配JSON规则配置
3. 移除已删除的接口实现

### 测试工作
1. 编写新的数据库访问层测试
2. 更新集成测试以适配新的数据库结构
3. 验证JSON规则配置的正确性

## 注意事项

1. 数据库已重新执行，现有数据可能需要重新导入
2. 后端代码需要相应调整以适配新的表结构
3. 前端如果直接使用相关API，需要更新接口调用