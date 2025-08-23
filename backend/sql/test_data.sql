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
INSERT INTO rule_templates (id, template_name, description, rules_config) 
VALUES 
('template-001', '经典模式', '经典的Royale Arena游戏模式', 
  '{"game_flow":{"day_duration":600,"night_duration":600},"map":{"places":["码头","工厂","贫民窟","旅馆","教堂","市政厅","消防局","池塘","住宅区","灯塔","小巷","学校","隧道","山道","寺庙","靶场","医院","森林","海滩","墓园","井","研究中心"]},"player":{"max_life":100,"max_strength":100,"daily_strength_recovery":40},"action":{"move_cost":5,"search_cost":5,"search_cooldown":30},"rest_mode":{"life_recovery":25,"max_moves":1},"teammate_behavior":0}'),
('template-002', '快速模式', '快节奏的游戏模式', 
  '{"game_flow":{"day_duration":300,"night_duration":300},"map":{"places":["码头","工厂","贫民窟","旅馆","教堂","市政厅","消防局","池塘","住宅区","灯塔","小巷","学校","隧道","山道","寺庙","靶场","医院","森林","海滩","墓园","井","研究中心"]},"player":{"max_life":100,"max_strength":100,"daily_strength_recovery":40},"action":{"move_cost":5,"search_cost":5,"search_cooldown":30},"rest_mode":{"life_recovery":25,"max_moves":1},"teammate_behavior":0}');

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