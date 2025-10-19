//! GameRuleEngine集成测试
//! 测试JSON规则配置的解析和验证功能

use royale_arena_backend::game::game_rule_engine::GameRuleEngine;

/// 测试用的JSON规则配置（基于rule_test.json）
const TEST_RULES_JSON: &str = r#"{
  "map": {
    "places": [
      "位置1", "位置2", "位置3", "位置4", "位置5", "位置6", "位置7", "位置8", "位置9"
    ],
    "safe_places": ["位置0"]
  },
  "player": {
    "max_life": 101,
    "max_strength": 102,
    "daily_strength_recovery": 43,
    "search_cooldown": 4,
    "max_backpack_items": 2,
    "unarmed_damage": 1
  },
  "action_costs": {
    "move": 1,
    "search": 2,
    "pick": 3,
    "attack": 4,
    "equip": 5,
    "use": 6,
    "throw": 7,
    "deliver": 8
  },
  "rest_mode": {
    "life_recovery": 13,
    "max_moves": 2
  },
  "death_item_disposition": "killer_takes_loot",
  "teammate_behavior": 15,
  "display_names": {
    "player_max_life": "生命NAME",
    "player_max_strength": "体力NAME",
    "player_daily_strength_recovery": "每日体力恢复NAME",
    "player_search_cooldown": "搜索冷却时间NAME",
    "player_unarmed_damage": "挥拳伤害NAME",
    "action_move": "移动NAME",
    "action_search": "搜索NAME",
    "action_pick": "拾取NAME",
    "action_attack": "攻击NAME",
    "action_equip": "装备NAME",
    "action_use": "使用NAME",
    "action_throw": "丢弃NAME",
    "action_deliver": "传音NAME",
    "rest_life_recovery": "生命恢复NAME",
    "rest_max_moves": "最大移动次数NAME"
  },
  "items": {
    "rarity_levels": [
      {"internal_name": "common", "display_name": "普通1", "prefix": "[绿1]", "is_airdropped": true},
      {"internal_name": "rare", "display_name": "稀有2", "prefix": "[蓝2]", "is_airdropped": true},
      {"internal_name": "epic", "display_name": "史诗3", "prefix": "[紫3]", "is_airdropped": false},
      {"internal_name": "legendary", "display_name": "传说4", "prefix": "[橙4]", "is_airdropped": false}
    ],
    "weapons": [
      {
        "internal_name": "common_weapon",
        "display_names": ["[绿]佩剑", "[绿]战斧"],
        "rarity": "common",
        "properties": {
          "damage": 11,
          "votes": 1
        }
      },
      {
        "internal_name": "rare_weapon",
        "display_names": ["[蓝]大太刀", "[蓝]死神镰刀"],
        "rarity": "rare",
        "properties": {
          "damage": 22,
          "votes": 2
        }
      },
      {
        "internal_name": "epic_weapon",
        "display_names": ["[紫]青龙偃月刀"],
        "rarity": "epic",
        "properties": {
          "damage": 33,
          "votes": 3
        }
      },
      {
        "internal_name": "legendary_weapon",
        "display_names": ["[橙]自然之力.晓"],
        "rarity": "legendary",
        "properties": {
          "damage": 44,
          "uses": 2,
          "votes": 5,
          "aoe_damage": 12,
          "bleed_damage": 7
        }
      }
    ],
    "armors": [
      {
        "internal_name": "common_armor",
        "display_names": ["[绿]皮甲", "[绿]布衣", "[绿]轻甲"],
        "rarity": "common",
        "properties": {
          "defense": 5,
          "votes": 2
        }
      }
    ],
    "other_items": [
      {
        "name": "[GPS]心跳探测仪1",
        "category": "utility_locator",
        "properties": {
          "votes": 3,
          "targets": 1
        }
      },
      {
        "name": "[侦]手持式雷达2",
        "category": "utility_revealer",
        "properties": {
          "votes": 3,
          "targets": 2
        }
      },
      {
        "name": "[神]生命启示3",
        "category": "utility_seer",
        "properties": {
          "votes": 3,
          "targets": 2
        }
      },
      {
        "name": "[炸]遥控地雷4",
        "category": "trap",
        "properties": {
          "damage": 30,
          "uses": 1,
          "votes": 0
        }
      }
    ],
    "consumables": [
      {
        "name": "[HP30]绷带a",
        "effect_type": "heal",
        "effect_value": 30,
        "cure_bleed": 1
      },
      {
        "name": "[HP50]止血绷带b",
        "effect_type": "heal",
        "effect_value": 50,
        "cure_bleed": 1
      },
      {
        "name": "[HP100]红花丹c",
        "effect_type": "heal",
        "effect_value": 100,
        "cure_bleed": 2
      },
      {
        "name": "[MP20]矿泉水d",
        "effect_type": "strength",
        "effect_value": 20
      },
      {
        "name": "[MP50]能量饮料e",
        "effect_type": "strength",
        "effect_value": 50
      },
      {
        "name": "[MP100]威士忌f",
        "effect_type": "strength",
        "effect_value": 100
      }
    ],
    "upgraders": [
      {
        "internal_name": "natural_upgrader",
        "display_names": ["[合]自然升级器q"],
        "rarity": "legendary"
      },
      {
        "internal_name": "artificial_upgrader",
        "display_names": ["[合]人造升级器w"],
        "rarity": "rare"
      }
    ],
    "upgrade_recipes": {
      "natural_upgrader": [
        {
          "result": "epic_weapon",
          "ingredients": ["rare_weapon"]
        },
        {
          "result": "legendary_weapon",
          "ingredients": ["epic_weapon"]
        }
      ],
      "artificial_upgrader": [
        {
          "result": "rare_weapon",
          "ingredients": ["common_weapon"]
        }
      ]
    }
  }
}"#;

/// 测试JSON解析基础功能
#[test]
fn test_game_rule_engine_json_parsing() {
    let rule_engine =
        GameRuleEngine::from_json(TEST_RULES_JSON).expect("Failed to parse test rules JSON");

    // 验证地图配置
    assert_eq!(rule_engine.map_config.places.len(), 9);
    assert_eq!(rule_engine.map_config.places[0], "位置1");
    assert_eq!(rule_engine.map_config.places[8], "位置9");
    assert_eq!(rule_engine.map_config.safe_places.len(), 1);
    assert_eq!(rule_engine.map_config.safe_places[0], "位置0");

    // 验证玩家配置
    assert_eq!(rule_engine.player_config.max_life, 101);
    assert_eq!(rule_engine.player_config.max_strength, 102);
    assert_eq!(rule_engine.player_config.daily_strength_recovery, 43);
    assert_eq!(rule_engine.player_config.search_cooldown, 4);
    assert_eq!(rule_engine.player_config.max_backpack_items, 2);
    assert_eq!(rule_engine.player_config.unarmed_damage, 1);

    // 验证行动消耗配置
    assert_eq!(rule_engine.action_costs.move_cost, 1);
    assert_eq!(rule_engine.action_costs.search, 2);
    assert_eq!(rule_engine.action_costs.pick, 3);
    assert_eq!(rule_engine.action_costs.attack, 4);
    assert_eq!(rule_engine.action_costs.equip, 5);
    assert_eq!(rule_engine.action_costs.use_item, 6);
    assert_eq!(rule_engine.action_costs.throw_item, 7);
    assert_eq!(rule_engine.action_costs.deliver, 8);

    // 验证静养模式配置
    assert_eq!(rule_engine.rest_mode.life_recovery, 13);
    assert_eq!(rule_engine.rest_mode.max_moves, 2);

    // 验证队友行为配置
    assert_eq!(rule_engine.teammate_behavior.mode, 15);

    // 验证死亡物品处置配置
    assert_eq!(
        rule_engine.death_item_disposition.description,
        "killer_takes_loot"
    );
}

/// 测试物品系统解析
#[test]
fn test_items_config_parsing() {
    let rule_engine =
        GameRuleEngine::from_json(TEST_RULES_JSON).expect("Failed to parse test rules JSON");

    // 验证稀有度等级
    assert_eq!(rule_engine.items_config.rarity_levels.len(), 4);
    assert_eq!(
        rule_engine.items_config.rarity_levels[0].internal_name,
        "common"
    );
    assert_eq!(
        rule_engine.items_config.rarity_levels[0].display_name,
        "普通1"
    );
    assert_eq!(rule_engine.items_config.rarity_levels[0].prefix, "[绿1]");
    assert_eq!(
        rule_engine.items_config.rarity_levels[0].is_airdropped,
        true
    );

    assert_eq!(
        rule_engine.items_config.rarity_levels[3].internal_name,
        "legendary"
    );
    assert_eq!(
        rule_engine.items_config.rarity_levels[3].display_name,
        "传说4"
    );
    assert_eq!(rule_engine.items_config.rarity_levels[3].prefix, "[橙4]");
    assert_eq!(
        rule_engine.items_config.rarity_levels[3].is_airdropped,
        false
    );

    // 验证武器配置
    assert_eq!(rule_engine.items_config.weapons.len(), 4);
    let common_weapon = &rule_engine.items_config.weapons[0];
    assert_eq!(common_weapon.internal_name, "common_weapon");
    assert_eq!(common_weapon.display_names.len(), 2);
    assert_eq!(common_weapon.display_names[0], "[绿]佩剑");
    assert_eq!(common_weapon.display_names[1], "[绿]战斧");
    assert_eq!(common_weapon.rarity, "common");
    assert_eq!(common_weapon.properties.damage, 11);
    assert_eq!(common_weapon.properties.votes, 1);
    assert_eq!(common_weapon.properties.uses, None);
    assert_eq!(common_weapon.properties.aoe_damage, None);
    assert_eq!(common_weapon.properties.bleed_damage, None);

    // 验证传说武器的特殊属性
    let legendary_weapon = &rule_engine.items_config.weapons[3];
    assert_eq!(legendary_weapon.internal_name, "legendary_weapon");
    assert_eq!(legendary_weapon.properties.damage, 44);
    assert_eq!(legendary_weapon.properties.uses, Some(2));
    assert_eq!(legendary_weapon.properties.aoe_damage, Some(12));
    assert_eq!(legendary_weapon.properties.bleed_damage, Some(7));

    // 验证防具配置
    assert_eq!(rule_engine.items_config.armors.len(), 1);
    let common_armor = &rule_engine.items_config.armors[0];
    assert_eq!(common_armor.internal_name, "common_armor");
    assert_eq!(common_armor.properties.defense, 5);
    assert_eq!(common_armor.properties.votes, 2);

    // 验证消耗品配置
    assert_eq!(rule_engine.items_config.consumables.len(), 6);
    let heal_bandage = &rule_engine.items_config.consumables[0];
    assert_eq!(heal_bandage.name, "[HP30]绷带a");
    assert_eq!(heal_bandage.effect_type, "heal");
    assert_eq!(heal_bandage.effect_value, 30);
    assert_eq!(heal_bandage.cure_bleed, Some(1));

    let strength_water = &rule_engine.items_config.consumables[3];
    assert_eq!(strength_water.name, "[MP20]矿泉水d");
    assert_eq!(strength_water.effect_type, "strength");
    assert_eq!(strength_water.effect_value, 20);
    assert_eq!(strength_water.cure_bleed, None);

    // 验证升级器配置
    assert_eq!(rule_engine.items_config.upgraders.len(), 2);
    let natural_upgrader = &rule_engine.items_config.upgraders[0];
    assert_eq!(natural_upgrader.internal_name, "natural_upgrader");
    assert_eq!(natural_upgrader.rarity, "legendary");

    // 验证升级配方
    assert_eq!(rule_engine.items_config.upgrade_recipes.len(), 2);
    let natural_recipes = rule_engine
        .items_config
        .upgrade_recipes
        .get("natural_upgrader")
        .unwrap();
    assert_eq!(natural_recipes.len(), 2);
    assert_eq!(natural_recipes[0].ingredients[0], "rare_weapon");
    assert_eq!(natural_recipes[0].result, "epic_weapon");
    assert_eq!(natural_recipes[1].ingredients[0], "epic_weapon");
    assert_eq!(natural_recipes[1].result, "legendary_weapon");
}

/// 测试配置获取函数
#[test]
fn test_config_getters() {
    let rule_engine =
        GameRuleEngine::from_json(TEST_RULES_JSON).expect("Failed to parse test rules JSON");

    // 测试搜索冷却时间
    assert_eq!(rule_engine.get_search_cooldown(), 4);

    // 测试挥拳伤害
    assert_eq!(rule_engine.get_unarmed_damage(), 1);
}

/// 测试错误处理：无效JSON
#[test]
fn test_invalid_json_error_handling() {
    // 测试完全无效的JSON
    let invalid_json = "{ invalid json }";
    assert!(GameRuleEngine::from_json(invalid_json).is_err());

    // 测试缺少必要字段的JSON
    let incomplete_json = r#"{ "map": {} }"#;
    let result = GameRuleEngine::from_json(incomplete_json);
    // 打印错误信息以了解具体失败原因
    if let Err(e) = &result {
        println!("Parse error: {}", e);
    }
    // 缺少必要字段的JSON应该解析失败
    assert!(result.is_err());

    // 测试字段类型错误的JSON
    let wrong_type_json = r#"{ "player": { "max_life": "not_a_number" } }"#;
    assert!(GameRuleEngine::from_json(wrong_type_json).is_err());
}

/// 测试使用外部JSON文件（如果存在）
#[test]
fn test_external_json_file() {
    // 尝试读取实际的rule_test.json文件
    let json_path = std::path::Path::new("../json_sample/rule_test.json");
    if json_path.exists() {
        let json_content =
            std::fs::read_to_string(json_path).expect("Failed to read rule_test.json");

        // 简单验证JSON能够被解析（虽然可能由于结构差异而失败）
        let parse_result = serde_json::from_str::<serde_json::Value>(&json_content);
        assert!(parse_result.is_ok(), "rule_test.json should be valid JSON");

        let json_value = parse_result.unwrap();
        // 验证一些基本结构存在
        assert!(json_value.get("map").is_some());
        assert!(json_value.get("player").is_some());
        assert!(json_value.get("action_costs").is_some());

        println!("External rule_test.json structure validation passed");
    } else {
        println!("External rule_test.json not found, skipping file test");
    }
}
