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
SET rules_config = '{\"map\":{\"places\":[\"码头\",\"工厂\",\"贫民窟\",\"旅馆\",\"教堂\",\"市政厅\",\"消防局\",\"池塘\",\"住宅区\",\"灯塔\",\"小巷\",\"学校\",\"隧道\",\"山道\",\"寺庙\",\"靶场\",\"医院\",\"森林\",\"海滩\",\"墓园\",\"井\",\"研究中心\"],\"safe_places\":[\"研究中心\"]},\"player\":{\"max_life\":100,\"max_strength\":100,\"daily_strength_recovery\":40,\"search_cooldown\":30,\"max_backpack_items\":6,\"unarmed_damage\":5},\"action_costs\":{\"move\":5,\"search\":5,\"pick\":0,\"attack\":0,\"equip\":0,\"use\":0,\"throw\":0,\"deliver\":10},\"rest_mode\":{\"life_recovery\":25,\"max_moves\":1},\"death_item_disposition\":\"killer_takes_loot\",\"teammate_behavior\":0,\"display_names\":{\"player_max_life\":\"生命值\",\"player_max_strength\":\"体力值\",\"player_daily_strength_recovery\":\"每日体力恢复\",\"player_search_cooldown\":\"搜索冷却时间\",\"player_unarmed_damage\":\"挥拳伤害\",\"action_move\":\"移动\",\"action_search\":\"搜索\",\"action_pick\":\"拾取\",\"action_attack\":\"攻击\",\"action_equip\":\"装备\",\"action_use\":\"使用\",\"action_throw\":\"丢弃\",\"action_deliver\":\"传音\",\"rest_life_recovery\":\"生命恢复\",\"rest_max_moves\":\"最大移动次数\"},\"items\":{\"rarity_levels\":[{\"internal_name\":\"common\",\"display_name\":\"普通\",\"prefix\":\"[绿]\",\"is_airdropped\":true},{\"internal_name\":\"rare\",\"display_name\":\"稀有\",\"prefix\":\"[蓝]\",\"is_airdropped\":true},{\"internal_name\":\"epic\",\"display_name\":\"史诗\",\"prefix\":\"[紫]\",\"is_airdropped\":false},{\"internal_name\":\"legendary\",\"display_name\":\"传说\",\"prefix\":\"[橙]\",\"is_airdropped\":false}],\"weapons\":[{\"internal_name\":\"common_weapon\",\"display_names\":[\"[绿]佩剑\",\"[绿]战斧\",\"[绿]长矛\",\"[绿]皮鞭\",\"[绿]回力镖\",\"[绿]IM-10\",\"[绿]复合弓\",\"[绿]铁爪\"],\"rarity\":\"common\",\"properties\":{\"damage\":10,\"votes\":1}},{\"internal_name\":\"rare_weapon\",\"display_names\":[\"[蓝]大太刀\",\"[蓝]死神镰刀\",\"[蓝]斩马刀\"],\"rarity\":\"rare\",\"properties\":{\"damage\":20,\"votes\":2}},{\"internal_name\":\"epic_weapon\",\"display_names\":[\"[紫]青龙偃月刀\",\"[紫]盘古斧\"],\"rarity\":\"epic\",\"properties\":{\"damage\":35,\"votes\":3}},{\"internal_name\":\"legendary_weapon\",\"display_names\":[\"[橙]自然之力.晓\",\"[橙]自然之力.夜\"],\"rarity\":\"legendary\",\"properties\":{\"damage\":50,\"uses\":5,\"votes\":0,\"aoe_damage\":40,\"bleed_damage\":10}}],\"armors\":[{\"internal_name\":\"common_armor\",\"display_names\":[\"[绿]皮甲\",\"[绿]布衣\",\"[绿]轻甲\"],\"rarity\":\"common\",\"properties\":{\"defense\":5,\"votes\":2}},{\"internal_name\":\"rare_armor\",\"display_names\":[\"[蓝]锁子甲\",\"[蓝]鳞甲\"],\"rarity\":\"rare\",\"properties\":{\"defense\":10,\"votes\":2}},{\"internal_name\":\"epic_armor\",\"display_names\":[\"[紫]板甲\",\"[紫]重甲\"],\"rarity\":\"epic\",\"properties\":{\"defense\":15,\"votes\":3}},{\"internal_name\":\"legendary_armor\",\"display_names\":[\"[橙]神佑之铠\",\"[橙]不朽战甲\"],\"rarity\":\"legendary\",\"properties\":{\"defense\":25,\"uses\":3,\"votes\":0}}],\"other_items\":[{\"name\":\"[GPS]心跳探测仪\",\"category\":\"utility_locator\",\"properties\":{\"votes\":3,\"targets\":1}},{\"name\":\"[侦]手持式雷达\",\"category\":\"utility_revealer\",\"properties\":{\"votes\":3,\"targets\":2}},{\"name\":\"[神]生命启示\",\"category\":\"utility_seer\",\"properties\":{\"votes\":3,\"targets\":2}},{\"name\":\"[炸]遥控地雷\",\"category\":\"trap\",\"properties\":{\"damage\":30,\"uses\":1,\"votes\":0}}],\"consumables\":[{\"name\":\"[HP30]绷带\",\"effect_type\":\"heal\",\"effect_value\":30,\"cure_bleed\":1},{\"name\":\"[HP50]止血绷带\",\"effect_type\":\"heal\",\"effect_value\":50,\"cure_bleed\":1},{\"name\":\"[HP100]红花丹\",\"effect_type\":\"heal\",\"effect_value\":100,\"cure_bleed\":2},{\"name\":\"[MP20]矿泉水\",\"effect_type\":\"strength\",\"effect_value\":20},{\"name\":\"[MP50]能量饮料\",\"effect_type\":\"strength\",\"effect_value\":50},{\"name\":\"[MP100]威士忌\",\"effect_type\":\"strength\",\"effect_value\":100}],\"upgraders\":[{\"internal_name\":\"natural_upgrader\",\"display_names\":[\"[合]自然升级器\"],\"rarity\":\"legendary\"},{\"internal_name\":\"artificial_upgrader\",\"display_names\":[\"[合]人造升级器\"],\"rarity\":\"rare\"}],\"upgrade_recipes\":{\"natural_upgrader\":[{\"result\":\"rare_weapon\",\"ingredients\":[\"common_weapon\"]},{\"result\":\"epic_weapon\",\"ingredients\":[\"rare_weapon\"]},{\"result\":\"legendary_weapon\",\"ingredients\":[\"epic_weapon\"]}],\"artificial_upgrader\":[{\"result\":\"rare_weapon\",\"ingredients\":[\"common_weapon\"]},{\"result\":\"epic_weapon\",\"ingredients\":[\"rare_weapon\"]}]}}}'
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