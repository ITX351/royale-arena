-- 该文件用于撤销初始化数据库结构的迁移
-- 由于是初始化脚本，通常在测试环境中不需要复杂的回滚逻辑
-- 可以简单地删除所有表（如果存在）

DROP TABLE IF EXISTS kill_records;
DROP TABLE IF EXISTS game_logs;
DROP TABLE IF EXISTS actors;
DROP TABLE IF EXISTS games;
DROP TABLE IF EXISTS rule_templates;
DROP TABLE IF EXISTS admin_users;