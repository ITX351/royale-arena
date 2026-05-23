-- migrations/20250825105153_init_schema.up.sql
-- 注意：不包含 DROP DATABASE, CREATE DATABASE, USE

-- 1. 管理员账户表
CREATE TABLE IF NOT EXISTS admin_users (
    id VARCHAR(36) PRIMARY KEY COMMENT '管理员唯一标识符(UUID)',
    username VARCHAR(50) NOT NULL UNIQUE COMMENT '管理员用户名',
    password VARCHAR(255) NOT NULL COMMENT '管理员密码(密文存储)',
    is_super_admin BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否为超级管理员',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    INDEX idx_admin_users_username (username)
) COMMENT '管理员账户表';

-- 2. 游戏规则模版表
CREATE TABLE IF NOT EXISTS rule_templates (
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

-- 3. 游戏实例表
CREATE TABLE IF NOT EXISTS games (
    id VARCHAR(36) PRIMARY KEY COMMENT '游戏唯一标识符(UUID)',
    name VARCHAR(100) NOT NULL COMMENT '游戏名称',
    description TEXT COMMENT '游戏描述',
    director_password VARCHAR(50) NOT NULL COMMENT '导演密码(1-40字符)',
    max_players INT NOT NULL DEFAULT 100 COMMENT '最大玩家数量',
    rules_config JSON NOT NULL COMMENT '游戏规则配置(JSON格式)',
    status ENUM('waiting', 'running', 'paused', 'ended','hidden','deleted') NOT NULL DEFAULT 'waiting' COMMENT '游戏状态',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX idx_games_status (status)
) COMMENT '游戏实例表';

-- 4. 演员账户表
CREATE TABLE IF NOT EXISTS actors (
    id VARCHAR(36) PRIMARY KEY COMMENT '演员唯一标识符(UUID)',
    game_id VARCHAR(36) NOT NULL COMMENT '所属游戏ID',
    name VARCHAR(50) NOT NULL COMMENT '演员名称',
    password VARCHAR(50) NOT NULL COMMENT '演员密码(1-40位字母数字)',
    team_id INT NOT NULL DEFAULT 0 COMMENT '队伍ID，用于标识玩家所属队伍，0表示无队伍',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    
    INDEX idx_actors_game_id (game_id),
    INDEX idx_actors_name (name)
) COMMENT '演员账户表';

-- 5. 游戏日志表
CREATE TABLE IF NOT EXISTS game_logs (
    `id` varchar(36) PRIMARY KEY COMMENT '日志唯一标识符(UUID)',
    `game_id` varchar(36) NOT NULL COMMENT '所属游戏ID',
    `type` enum('SystemNotice','UserDirected') NOT NULL COMMENT '日志级别',
    `message` text NOT NULL COMMENT '日志消息',
    `player_id` varchar(36) NULL COMMENT '相关玩家ID',
    `timestamp` timestamp DEFAULT CURRENT_TIMESTAMP COMMENT '日志时间戳',
    `visible_to_all_players` BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否对所有玩家可见',
    `visible_to_director` BOOLEAN NOT NULL DEFAULT TRUE COMMENT '是否对导演可见',
    FOREIGN KEY (`game_id`) REFERENCES games(id) ON DELETE CASCADE,
    FOREIGN KEY (`player_id`) REFERENCES actors(id) ON DELETE CASCADE,
    
    INDEX idx_game_logs_game_id (game_id),
    INDEX idx_game_logs_type (type),
    INDEX idx_game_logs_timestamp (timestamp),
    INDEX idx_game_logs_player_id (player_id),
    INDEX idx_game_logs_visible_to_all_players (visible_to_all_players),
    INDEX idx_game_logs_visible_to_director (visible_to_director)
) COMMENT '游戏日志表';

-- 6. 击杀记录表
CREATE TABLE IF NOT EXISTS kill_records (
    id VARCHAR(36) PRIMARY KEY COMMENT '击杀记录唯一标识符(UUID)',
    game_id VARCHAR(36) NOT NULL COMMENT '所属游戏ID',
    killer_id VARCHAR(36) NULL COMMENT '击杀者ID（可为空，表示非玩家击杀）',
    victim_id VARCHAR(36) NOT NULL COMMENT '被击杀者ID',
    kill_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '击杀时间',
    cause VARCHAR(50) NOT NULL COMMENT '击杀原因（如：武器、缩圈等）',
    weapon VARCHAR(50) NULL COMMENT '使用的武器/方式',
    location VARCHAR(100) NULL COMMENT '击杀地点',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    FOREIGN KEY (killer_id) REFERENCES actors(id) ON DELETE CASCADE,
    FOREIGN KEY (victim_id) REFERENCES actors(id) ON DELETE CASCADE,
    
    INDEX idx_kill_records_game_id (game_id),
    INDEX idx_kill_records_killer_id (killer_id),
    INDEX idx_kill_records_victim_id (victim_id),
    INDEX idx_kill_records_kill_time (kill_time)
) COMMENT '击杀记录表';