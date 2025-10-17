-- 为game_logs表添加新字段
ALTER TABLE game_logs 
ADD COLUMN visible_to_all_players BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN visible_to_director BOOLEAN NOT NULL DEFAULT TRUE;

-- 为新字段创建索引
CREATE INDEX idx_game_logs_visible_to_all_players ON game_logs(visible_to_all_players);
CREATE INDEX idx_game_logs_visible_to_director ON game_logs(visible_to_director);
