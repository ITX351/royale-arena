-- Royale Arena 测试数据插入脚本
-- 用于初始化测试环境的数据

USE royale_arena;

-- 插入测试管理员账户
-- 管理员a (密码: 1)
INSERT INTO admin_users (id, username, password, is_super_admin) 
VALUES ('3e6aa7c8-197a-4627-8ead-0afb2f050677', 'a', '$2b$12$j/bJMGgwyj0tzDSRHZWQZuMPqwOdgCRDMQLO6Sle/ONIMcDMXE15e', TRUE);

-- 管理员b (密码: 2)
INSERT INTO admin_users (id, username, password, is_super_admin) 
VALUES ('00f2bd7d-99f8-4bc7-a4a8-f2293549f726', 'b', '$2b$12$xQn6ti4eKhH4H3MXn2OdHufT1HWv.NZOI2rVizjjPYfLkDFKt7zFC', FALSE);

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
INSERT INTO actors (id, game_id, name, password, team_id) 
VALUES 
('actor-001', 'game-001', '演员1', 'actor1', 0),
('actor-002', 'game-001', '演员2', 'actor2', 0),
('actor-003', 'game-002', '演员3', 'actor3', 0),
('actor-004', 'game-002', '演员4', 'actor4', 0);

-- 插入初始游戏统计数据
INSERT INTO game_stats (id, game_id, player_count, alive_players) 
VALUES 
('stats-001', 'game-001', 2, 2),
('stats-002', 'game-002', 2, 2);