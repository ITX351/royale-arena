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
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间'
) COMMENT '管理员账户表';

-- 2. 游戏规则模版表
-- 存储可重用的游戏规则配置模版
CREATE TABLE rule_templates (
    id VARCHAR(36) PRIMARY KEY COMMENT '模版唯一标识符(UUID)',
    template_name VARCHAR(100) NOT NULL UNIQUE COMMENT '模版名称',
    description TEXT COMMENT '模版描述',
    -- 游戏流程配置
    day_duration INT NULL COMMENT '白天时长(秒)，NULL表示使用默认值',
    night_duration INT NULL COMMENT '夜晚时长(秒)，NULL表示使用默认值',
    -- 地图配置
    places JSON NULL COMMENT '地点列表(JSON数组)，NULL表示使用默认地图',
    -- 玩家配置
    max_life INT NULL COMMENT '最大生命值，NULL表示使用默认值',
    max_strength INT NULL COMMENT '最大体力值，NULL表示使用默认值',
    daily_strength_recovery INT NULL COMMENT '每日体力恢复值，NULL表示使用默认值',
    -- 行动配置
    move_cost INT NULL COMMENT '移动消耗体力，NULL表示使用默认值',
    search_cost INT NULL COMMENT '搜索消耗体力，NULL表示使用默认值',
    search_cooldown INT NULL COMMENT '搜索冷却时间(秒)，NULL表示使用默认值',
    -- 静养模式配置
    life_recovery INT NULL COMMENT '静养模式生命恢复值，NULL表示使用默认值',
    max_moves INT NULL COMMENT '静养模式最大移动次数，NULL表示使用默认值',
    -- 队友行为规则
    teammate_behavior INT NOT NULL DEFAULT 0 COMMENT '队友行为规则，位压缩存储：0-无限制，1-禁止队友伤害，2-禁止搜索到队友，4-允许观看队友状态，8-允许赠送队友物品',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '模版创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '模版更新时间'
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
    FOREIGN KEY (rule_template_id) REFERENCES rule_templates(id) ON DELETE SET NULL
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
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
) COMMENT '演员账户表';

-- 5. 游戏统计数据表
-- 存储游戏的统计数据
CREATE TABLE game_stats (
    id VARCHAR(36) PRIMARY KEY COMMENT '统计数据唯一标识符(UUID)',
    game_id VARCHAR(36) NOT NULL COMMENT '所属游戏ID',
    player_count INT NOT NULL DEFAULT 0 COMMENT '玩家总数',
    alive_players INT NOT NULL DEFAULT 0 COMMENT '存活玩家数',
    total_actions INT NOT NULL DEFAULT 0 COMMENT '总行动数',
    start_time TIMESTAMP NULL COMMENT '游戏开始时间',
    duration INT NOT NULL DEFAULT 0 COMMENT '游戏持续时间(秒)',
    votes_cast INT NOT NULL DEFAULT 0 COMMENT '总投票数',
    eliminations INT NOT NULL DEFAULT 0 COMMENT '淘汰数',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
) COMMENT '游戏统计数据表';

-- 6. 游戏日志表
-- 存储游戏运行过程中的日志信息
CREATE TABLE game_logs (
    id VARCHAR(36) PRIMARY KEY COMMENT '日志唯一标识符(UUID)',
    game_id VARCHAR(36) NOT NULL COMMENT '所属游戏ID',
    level ENUM('info', 'warn', 'error') NOT NULL COMMENT '日志级别',
    message TEXT NOT NULL COMMENT '日志消息',
    player_id VARCHAR(36) NULL COMMENT '相关玩家ID',
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '日志时间戳',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    FOREIGN KEY (player_id) REFERENCES actors(id) ON DELETE SET NULL
) COMMENT '游戏日志表';

-- 7. 投票记录表
-- 存储玩家的投票记录
CREATE TABLE votes (
    id VARCHAR(36) PRIMARY KEY COMMENT '投票记录唯一标识符(UUID)',
    game_id VARCHAR(36) NOT NULL COMMENT '所属游戏ID',
    voter_id VARCHAR(36) NOT NULL COMMENT '投票者ID',
    target_id VARCHAR(36) NOT NULL COMMENT '被投票者ID',
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '投票时间',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    FOREIGN KEY (voter_id) REFERENCES actors(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES actors(id) ON DELETE CASCADE
) COMMENT '投票记录表';

-- 创建索引以提高查询性能
-- 管理员表索引
CREATE INDEX idx_admin_users_username ON admin_users(username);

-- 游戏规则模版表索引
CREATE INDEX idx_rule_templates_name ON rule_templates(template_name);

-- 游戏实例表索引
CREATE INDEX idx_games_status ON games(status);

-- 演员账户表索引
CREATE INDEX idx_actors_game_id ON actors(game_id);
CREATE INDEX idx_actors_name ON actors(name);

-- 游戏统计数据表索引
CREATE INDEX idx_game_stats_game_id ON game_stats(game_id);

-- 游戏日志表索引
CREATE INDEX idx_game_logs_game_id ON game_logs(game_id);
CREATE INDEX idx_game_logs_level ON game_logs(level);
CREATE INDEX idx_game_logs_timestamp ON game_logs(timestamp);

-- 投票记录表索引
CREATE INDEX idx_votes_game_id ON votes(game_id);
CREATE INDEX idx_votes_voter_id ON votes(voter_id);
CREATE INDEX idx_votes_target_id ON votes(target_id);
CREATE INDEX idx_votes_timestamp ON votes(timestamp);