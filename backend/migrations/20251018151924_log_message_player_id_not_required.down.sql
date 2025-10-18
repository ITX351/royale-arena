DELETE FROM `game_logs` WHERE `player_id` IS NULL;

ALTER TABLE `game_logs`
	MODIFY COLUMN `player_id` varchar(36) NOT NULL COMMENT '相关玩家ID';
