# 数据库表结构新增字段设计文档

## 概述

本文档描述了为`game_logs`表增加两个非空索引字段的需求，以及对日志服务和处理器的修改。注意：根据实际需求分析，修改的是`game_logs`表而非`kill_records`表。

## 数据库表结构变更

### game_logs 表结构修改

为`game_logs`表添加两个新的非空字段：
- `visible_to_all_players`: BOOLEAN类型，用于标识该日志记录是否向所有玩家展示
- `visible_to_director`: BOOLEAN类型，用于标识该日志记录是否向导演展示

同时为这两个字段创建索引以提高查询性能。

### 数据库迁移脚本

#### 正向迁移 (up)

文件: `backend/migrations/20251017154545_game_log_new_filter.up.sql`

```sql
-- 为game_logs表添加新字段
ALTER TABLE game_logs 
ADD COLUMN visible_to_all_players BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN visible_to_director BOOLEAN NOT NULL DEFAULT FALSE;

-- 为新字段创建索引
CREATE INDEX idx_game_logs_visible_to_all_players ON game_logs(visible_to_all_players);
CREATE INDEX idx_game_logs_visible_to_director ON game_logs(visible_to_director);
```

#### 回滚迁移 (down)

文件: `backend/migrations/20251017154545_game_log_new_filter.down.sql`

```sql
-- 删除索引
DROP INDEX idx_game_logs_visible_to_all_players ON game_logs;
DROP INDEX idx_game_logs_visible_to_director ON game_logs;

-- 删除字段
ALTER TABLE game_logs 
DROP COLUMN visible_to_all_players,
DROP COLUMN visible_to_director;
```

## 日志服务修改

### GameLogService 结构修改

1. `create_log`函数增加两个新参数：
   - `visible_to_all_players`: BOOLEAN，标识是否向所有玩家展示
   - `visible_to_director`: BOOLEAN，标识是否向导演展示

2. `get_player_messages`函数保持原有逻辑，但增加对`visible_to_all_players`字段的过滤：
   - 返回指定玩家相关的消息记录
   - 额外包含所有标记为`visible_to_all_players`为true的记录

3. 新增`get_director_messages`函数：
   - 不检测`player_id`
   - 只返回`visible_to_director`为true的记录

4. 新增`delete_logs_after_timestamp`函数：
   - 输入一个游戏ID和一个时间戳（Option类型）
   - 删除指定游戏ID晚于指定时间戳的数据（如果时间戳为空则删除全部数据）

### 数据模型修改

在`MessageRecord`模型中增加两个新字段：
- `visible_to_all_players`: BOOLEAN
- `visible_to_director`: BOOLEAN

## WebSocket服务修改

在WebSocket服务中调用`create_log`时，从`ActionResult`结构体中传递对应的参数值：
- `visible_to_all_players`对应`ActionResult.broadcast_to_all`
- `visible_to_director`对应`ActionResult.broadcast_to_director`

## API处理器修改

1. 在`handlers.rs`中新增一个路由用于导演查询日志记录：
   - 路径: `/games/{game_id}/director/logs`
   - 方法: GET
   - 查询参数: 导演密码
   - 调用新增的`get_director_messages`函数获取日志记录

2. 新增一个供管理员使用的路由（需要验证JWT）：
   - 路径: `/admin/games/{game_id}/logs`
   - 方法: DELETE
   - 调用`delete_logs_after_timestamp`函数删除指定游戏的所有日志记录

## 实现要点

1. 所有修改均为破坏性修改，不考虑向后兼容
2. 数据库迁移脚本遵循项目规范
3. 仅实现最小功能，不添加额外功能或测试