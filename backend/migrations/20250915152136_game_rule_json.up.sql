-- 添加新的规则配置字段（初始为NULL）
ALTER TABLE games 
ADD COLUMN rules_config JSON NULL 
COMMENT '游戏规则配置(JSON格式)' 
AFTER max_players;

-- 迁移现有数据：将关联的规则模板配置复制到游戏表中
-- 对于没有关联模板的游戏，设置默认规则配置
UPDATE games g 
INNER JOIN rule_templates rt ON g.rule_template_id = rt.id 
SET g.rules_config = rt.rules_config;

-- 为没有关联模板的游戏设置默认规则配置
UPDATE games 
SET rules_config = '{"game_flow":{"day_duration":600,"night_duration":600},"map":{"places":["码头","工厂","贫民窟","旅馆","教堂","市政厅","消防局","池塘","住宅区","灯塔","小巷","学校","隧道","山道","寺庙","靶场","医院","森林","海滩","墓园","井","研究中心"]},"player":{"max_life":100,"max_strength":100,"daily_strength_recovery":40},"action":{"move_cost":5,"search_cost":5,"search_cooldown":30},"rest_mode":{"life_recovery":25,"max_moves":1},"teammate_behavior":0}'
WHERE rules_config IS NULL;

-- 修改字段为NOT NULL
ALTER TABLE games 
MODIFY COLUMN rules_config JSON NOT NULL 
COMMENT '游戏规则配置(JSON格式)' 
AFTER max_players;

-- 删除外键约束
ALTER TABLE games 
DROP FOREIGN KEY games_ibfk_1;

-- 删除rule_template_id字段
ALTER TABLE games 
DROP COLUMN rule_template_id;