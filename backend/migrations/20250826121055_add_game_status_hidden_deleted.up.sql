-- migrations/20250826121055_add_game_status_hidden_deleted.up.sql
-- 为游戏状态添加 'hidden' 和 'deleted' 状态

-- 修改games表的status字段，添加新的枚举值
ALTER TABLE games MODIFY COLUMN status ENUM('waiting', 'running', 'paused', 'ended', 'hidden', 'deleted') NOT NULL DEFAULT 'waiting' COMMENT '游戏状态';

-- 添加索引以优化按状态查询的性能
DROP INDEX IF EXISTS idx_games_status ON games;
CREATE INDEX idx_games_status ON games(status);

-- 为了支持"全部"筛选（不包括已隐藏），添加一个视图可能有用，但这里先保持简单
-- 在应用层处理筛选逻辑