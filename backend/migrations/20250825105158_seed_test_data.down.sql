-- 该文件用于撤销填充测试数据的迁移
-- 可以通过删除所有数据来实现，或者根据需要更精细地操作

-- 删除所有测试数据
-- 注意：外键约束可能会要求按特定顺序删除
DELETE FROM kill_records;
DELETE FROM game_logs;
DELETE FROM actors;
DELETE FROM games;
DELETE FROM rule_templates;
DELETE FROM admin_users;

-- 或者，如果需要重置自增ID（对于有自增主键的表，虽然这里用的是UUID）
-- TRUNCATE TABLE kill_records;
-- TRUNCATE TABLE game_logs;
-- TRUNCATE TABLE actors;
-- TRUNCATE TABLE games;
-- TRUNCATE TABLE rule_templates;
-- TRUNCATE TABLE admin_users;