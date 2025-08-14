-- Royale Arena 测试数据插入脚本
-- 用于初始化测试环境的数据

USE royale_arena;

-- 插入测试管理员账户
-- 普通管理员 (密码: admin123)
INSERT INTO admin_users (id, username, password, is_super_admin) 
VALUES ('admin-001', 'admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.PZvO.S', FALSE);

-- 超级管理员 (密码: super123)
INSERT INTO admin_users (id, username, password, is_super_admin) 
VALUES ('super-001', 'superadmin', '$2b$12$V3l9YdlG7jwBn8pF4Hp9yO//.C.QQVU51RoX5h6mZ55Xv6z9.1.2C', TRUE);

-- 插入测试游戏规则模版
INSERT INTO rule_templates (id, template_name, description, day_duration, night_duration) 
VALUES 
('template-001', '经典模式', '经典的Royale Arena游戏模式', 600, 600),
('template-002', '快速模式', '快节奏的游戏模式', 300, 300);

-- 插入测试游戏
INSERT INTO games (id, name, description, director_password, max_players) 
VALUES 
('game-001', '测试游戏1', '用于测试的第一个游戏实例', 'director123', 50),
('game-002', '测试游戏2', '用于测试的第二个游戏实例', 'director456', 100);

-- 插入测试演员账户
INSERT INTO actors (id, game_id, name, password) 
VALUES 
('actor-001', 'game-001', '演员1', 'actor1'),
('actor-002', 'game-001', '演员2', 'actor2'),
('actor-003', 'game-002', '演员3', 'actor3'),
('actor-004', 'game-002', '演员4', 'actor4');

-- 插入初始游戏统计数据
INSERT INTO game_stats (id, game_id, player_count, alive_players) 
VALUES 
('stats-001', 'game-001', 2, 2),
('stats-002', 'game-002', 2, 2);