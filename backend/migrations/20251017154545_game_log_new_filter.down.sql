-- 删除索引
DROP INDEX idx_game_logs_visible_to_all_players ON game_logs;
DROP INDEX idx_game_logs_visible_to_director ON game_logs;

-- 删除字段
ALTER TABLE game_logs 
DROP COLUMN visible_to_all_players,
DROP COLUMN visible_to_director;
