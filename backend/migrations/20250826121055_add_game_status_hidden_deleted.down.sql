-- migrations/20250826121055_add_game_status_hidden_deleted.down.sql
-- 回滚游戏状态的 'hidden' 和 'deleted' 状态

-- 还原games表的status字段，移除hidden和deleted枚举值
ALTER TABLE games MODIFY COLUMN status ENUM('waiting', 'running', 'paused', 'ended') NOT NULL DEFAULT 'waiting' COMMENT '游戏状态';

-- 重建索引
DROP INDEX IF EXISTS idx_games_status ON games;
CREATE INDEX idx_games_status ON games(status);