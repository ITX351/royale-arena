
ALTER TABLE games MODIFY COLUMN status ENUM('waiting', 'running', 'paused', 'ended', 'hidden', 'deleted') NOT NULL DEFAULT 'waiting' COMMENT '游戏状态';
