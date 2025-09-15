-- 重新添加rule_template_id字段
ALTER TABLE games 
ADD COLUMN rule_template_id VARCHAR(36) NULL 
COMMENT '关联的规则模板ID' 
AFTER max_players;

-- 删除rules_config字段
ALTER TABLE games 
DROP COLUMN rules_config;

-- 重新创建外键约束
ALTER TABLE games 
ADD CONSTRAINT games_ibfk_1 
FOREIGN KEY (rule_template_id) REFERENCES rule_templates(id) 
ON DELETE SET NULL;