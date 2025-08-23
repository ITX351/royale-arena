-- Royale Arena 数据库初始化脚本
-- 数据库名称: royale_arena
-- 字符集: utf8mb4
-- 排序规则: utf8mb4_unicode_ci

-- 删除现有数据库（如果存在）
DROP DATABASE IF EXISTS royale_arena;

-- 创建数据库
CREATE DATABASE royale_arena 
    DEFAULT CHARACTER SET utf8mb4 
    DEFAULT COLLATE utf8mb4_unicode_ci;

-- 使用数据库
USE royale_arena;

-- 1. 管理员账户表
-- 存储系统管理员信息
CREATE TABLE admin_users (
    id VARCHAR(36) PRIMARY KEY COMMENT '管理员唯一标识符(UUID)',
    username VARCHAR(50) NOT NULL UNIQUE COMMENT '管理员用户名',
    password VARCHAR(255) NOT NULL COMMENT '管理员密码(密文存储)',
    is_super_admin BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否为超级管理员',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    
    INDEX idx_admin_users_username (username)
) COMMENT '管理员账户表';

-- 2. 游戏规则模版表
-- 存储可重用的游戏规则配置模版
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

-- 3. 游戏实例表
-- 存储每个游戏实例的基本信息
CREATE TABLE games (
    id VARCHAR(36) PRIMARY KEY COMMENT '游戏唯一标识符(UUID)',
    name VARCHAR(100) NOT NULL COMMENT '游戏名称',
    description TEXT COMMENT '游戏描述',
    director_password VARCHAR(50) NOT NULL COMMENT '导演密码',
    max_players INT NOT NULL DEFAULT 100 COMMENT '最大玩家数量',
    status ENUM('waiting', 'running', 'paused', 'ended') NOT NULL DEFAULT 'waiting' COMMENT '游戏状态',
    -- 关联规则模板
    rule_template_id VARCHAR(36) NULL COMMENT '关联的规则模板ID',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    FOREIGN KEY (rule_template_id) REFERENCES rule_templates(id) ON DELETE SET NULL,
    
    INDEX idx_games_status (status)
) COMMENT '游戏实例表';

-- 4. 演员账户表
-- 存储游戏中演员的信息
CREATE TABLE actors (
    id VARCHAR(36) PRIMARY KEY COMMENT '演员唯一标识符(UUID)',
    game_id VARCHAR(36) NOT NULL COMMENT '所属游戏ID',
    name VARCHAR(50) NOT NULL COMMENT '演员名称',
    password VARCHAR(8) NOT NULL COMMENT '演员密码(6-8位字母数字)',
    team_id INT NOT NULL DEFAULT 0 COMMENT '队伍ID，用于标识玩家所属队伍，0表示无队伍',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    
    INDEX idx_actors_game_id (game_id),
    INDEX idx_actors_name (name)
) COMMENT '演员账户表';

-- 5. 游戏日志表
-- 存储游戏运行过程中的日志信息
CREATE TABLE game_logs (
    id VARCHAR(36) PRIMARY KEY COMMENT '日志唯一标识符(UUID)',
    game_id VARCHAR(36) NOT NULL COMMENT '所属游戏ID',
    level ENUM('info', 'warn', 'error') NOT NULL COMMENT '日志级别',
    message TEXT NOT NULL COMMENT '日志消息',
    player_id VARCHAR(36) NULL COMMENT '相关玩家ID',
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '日志时间戳',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES actors(id) ON DELETE SET NULL,
    
    INDEX idx_game_logs_game_id (game_id),
    INDEX idx_game_logs_level (level),
    INDEX idx_game_logs_timestamp (timestamp)
) COMMENT '游戏日志表';

-- 5. 击杀记录表
-- 存储玩家的击杀记录
CREATE TABLE kill_records (
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