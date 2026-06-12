//! 货币系统和玩家移动功能集成测试
//! 测试新增的货币类型、货币使用、导演设置货币和导演移动玩家功能

use royale_arena_backend::game::game_rule_engine::{GameRuleEngine, ItemType};
use royale_arena_backend::game::models::MessageType;
use royale_arena_backend::websocket::actions::player_action_scheduler::ActionParams;
use royale_arena_backend::websocket::models::{GameState, Place, Player, ShopBuyItem, ShopListing};
use serde_json::{Value, json};

/// 测试规则配置（包含货币配置）
fn get_test_rules_with_currency() -> serde_json::Value {
    json!({
      "map": {
        "places": ["位置1", "位置2", "位置3"],
        "safe_places": ["位置0"]
      },
      "player": {
        "max_life": 100,
        "max_strength": 100,
        "daily_life_recovery": 10,
        "daily_strength_recovery": 20,
        "search_cooldown": 4,
        "max_backpack_items": 5,
        "unarmed_damage": 10
      },
      "action_costs": {
        "move": 5,
        "search": 10,
        "pick": 5,
        "attack": 15,
        "equip": 5,
        "use": 5,
        "throw": 5,
        "deliver": 10
      },
      "rest_mode": {
        "life_recovery": 20,
        "strength_recovery": 30,
        "max_moves": 3
      },
      "death_item_disposition": "killer_takes_loot",
      "teammate_behavior": 0,
      "display_names": {},
      "items_config": {
        "rarity_levels": [
          {"internal_name": "common", "display_name": "普通", "prefix": "[普]", "is_airdropped": true}
        ],
        "items": {
          "weapons": [
            {
              "internal_name": "test_weapon",
              "display_names": ["测试剑"],
              "rarity": "common",
              "properties": {
                "damage": 20,
                "votes": 1
              }
            }
          ],
          "armors": [],
          "utilities": [],
          "consumables": [
            {
              "name": "[HP10]测试药水",
              "properties": {
                "effect_type": "heal",
                "effect_value": 10
              }
            }
          ],
          "upgraders": [],
          "currencies": [
            {
              "name": "金币",
              "properties": {
                "value": 100
              }
            },
            {
              "name": "银币",
              "properties": {
                "value": 10
              }
            }
          ]
        }
      }
    })
}

fn get_test_rules_with_single_currency(name: &str, value: i32) -> Value {
    let mut rules = get_test_rules_with_currency();
    rules["items_config"]["items"]["currencies"] = json!([
        {
            "name": name,
            "properties": {
                "value": value
            }
        }
    ]);
    rules
}

fn add_test_place(game_state: &mut GameState, place_name: &str) {
    game_state
        .places
        .entry(place_name.to_string())
        .or_insert_with(|| Place::new(place_name.to_string()));
}

fn add_test_player(
    game_state: &mut GameState,
    player_id: &str,
    player_name: &str,
    location: &str,
    coins: i32,
) {
    add_test_place(game_state, location);

    let mut player = Player::new(
        player_id.to_string(),
        player_name.to_string(),
        "password".to_string(),
        1,
        &game_state.rule_engine,
    );
    player.location = location.to_string();
    player.coins = coins;

    game_state.players.insert(player_id.to_string(), player);
    game_state
        .places
        .get_mut(location)
        .unwrap()
        .players
        .push(player_id.to_string());
}

fn make_shop_listing(id: &str, item_name: &str, price: i32, quantity: i32) -> ShopListing {
    ShopListing {
        id: id.to_string(),
        item_name: item_name.to_string(),
        price,
        quantity,
    }
}

fn empty_action_params() -> ActionParams {
    ActionParams {
        target_place: None,
        place_name: None,
        item_id: None,
        slot_type: None,
        target_player_id: None,
        target_player_ids: None,
        target_item_name: None,
        message: None,
        shop_buy_items: None,
    }
}

/// 测试：货币配置解析
#[test]
fn test_currency_config_parsing() {
    let rules_json =
        serde_json::to_string(&get_test_rules_with_currency()).expect("Failed to serialize rules");
    let rule_engine =
        GameRuleEngine::from_json(&rules_json).expect("Failed to parse test rules with currency");

    // 验证货币配置存在
    let currencies = &rule_engine.items_config.items.currencies;
    assert_eq!(currencies.len(), 2, "应该有2个货币类型");

    // 验证金币
    assert_eq!(currencies[0].name, "金币");
    assert_eq!(currencies[0].properties.value, 100);

    // 验证银币
    assert_eq!(currencies[1].name, "银币");
    assert_eq!(currencies[1].properties.value, 10);
}

/// 测试：从配置创建货币物品
#[test]
fn test_create_currency_item_from_config() {
    let rules_json =
        serde_json::to_string(&get_test_rules_with_currency()).expect("Failed to serialize rules");
    let rule_engine =
        GameRuleEngine::from_json(&rules_json).expect("Failed to parse test rules with currency");

    // 创建金币物品
    let gold_coin = rule_engine
        .create_item_from_name("金币")
        .expect("Failed to create gold coin");

    assert_eq!(gold_coin.name, "金币");

    // 验证物品类型是货币
    match &gold_coin.item_type {
        ItemType::Currency(props) => {
            assert_eq!(props.value, 100, "金币面值应为100");
        }
        _ => panic!("物品应该是货币类型"),
    }

    // 创建银币物品
    let silver_coin = rule_engine
        .create_item_from_name("银币")
        .expect("Failed to create silver coin");

    match &silver_coin.item_type {
        ItemType::Currency(props) => {
            assert_eq!(props.value, 10, "银币面值应为10");
        }
        _ => panic!("物品应该是货币类型"),
    }
}

/// 测试：创建不存在的货币物品应该失败
#[test]
fn test_create_nonexistent_currency_item() {
    let rules_json =
        serde_json::to_string(&get_test_rules_with_currency()).expect("Failed to serialize rules");
    let rule_engine =
        GameRuleEngine::from_json(&rules_json).expect("Failed to parse test rules with currency");

    let result = rule_engine.create_item_from_name("不存在的货币");
    assert!(result.is_err(), "创建不存在的货币应该失败");
}

/// 测试：玩家初始化时货币为0
#[test]
fn test_player_initial_coins_is_zero() {
    let rules_json =
        serde_json::to_string(&get_test_rules_with_currency()).expect("Failed to serialize rules");
    let rule_engine =
        GameRuleEngine::from_json(&rules_json).expect("Failed to parse test rules with currency");

    let player = Player::new(
        "test_player_1".to_string(),
        "测试玩家".to_string(),
        "password".to_string(),
        1,
        &rule_engine,
    );

    assert_eq!(player.coins, 0, "新玩家初始货币应为0");
}

/// 测试：导演设置玩家货币
#[test]
fn test_director_set_player_coins() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_coins".to_string(), rules_json);

    let player_id = "test_player_coins";
    let mut player = Player::new(
        player_id.to_string(),
        "玩家C".to_string(),
        "password".to_string(),
        1,
        &game_state.rule_engine,
    );

    player.coins = 50;
    player.location = "位置1".to_string();

    game_state.players.insert(player_id.to_string(), player);
    game_state
        .places
        .insert("位置1".to_string(), Place::new("位置1".to_string()));

    // 导演设置玩家货币为100
    let result = game_state.handle_set_player_coins(player_id, 100);
    assert!(result.is_ok(), "导演设置货币应该成功");

    let updated_player = game_state.players.get(player_id).unwrap();
    assert_eq!(updated_player.coins, 100, "导演设置的货币值应该正确应用");

    // 导演设置玩家货币为当前值（应该返回成功）
    let result = game_state.handle_set_player_coins(player_id, 100);
    assert!(result.is_ok(), "设置相同值应该返回成功");

    // 导演设置玩家货币为0
    let result = game_state.handle_set_player_coins(player_id, 0);
    assert!(result.is_ok(), "设置货币为0应该成功");

    let updated_player = game_state.players.get(player_id).unwrap();
    assert_eq!(updated_player.coins, 0, "货币应该被设置为0");

    // 导演设置玩家货币为负数
    let result = game_state.handle_set_player_coins(player_id, -50);
    assert!(result.is_ok(), "可以设置负数货币");

    let updated_player = game_state.players.get(player_id).unwrap();
    assert_eq!(updated_player.coins, -50, "负数货币应该被正确设置");
}

/// 测试：导演设置不存在的玩家货币（应该失败）
#[test]
fn test_director_set_currency_nonexistent_player() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_noexist".to_string(), rules_json);

    let result = game_state.handle_set_player_coins("nonexistent_player", 100);
    assert!(result.is_err(), "设置不存在玩家的货币应该失败");
    assert_eq!(result.unwrap_err(), "Player not found");
}

/// 测试：导演移动玩家到不同位置
#[test]
fn test_director_move_player() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_move".to_string(), rules_json);

    let player_id = "test_player_move";
    let mut player = Player::new(
        player_id.to_string(),
        "玩家D".to_string(),
        "password".to_string(),
        1,
        &game_state.rule_engine,
    );

    player.location = "位置1".to_string();
    game_state.players.insert(player_id.to_string(), player);

    // 初始化两个位置
    game_state
        .places
        .insert("位置1".to_string(), Place::new("位置1".to_string()));
    game_state
        .places
        .insert("位置2".to_string(), Place::new("位置2".to_string()));

    // 导演将玩家从位置1移动到位置2
    let result = game_state.handle_move_player(player_id, "位置2");
    assert!(result.is_ok(), "导演移动玩家应该成功: {:?}", result.err());

    let updated_player = game_state.players.get(player_id).unwrap();
    assert_eq!(
        updated_player.location, "位置2",
        "玩家位置应该被更新为位置2"
    );
}

/// 测试：导演移动玩家到相同位置（应该返回成功但未变化）
#[test]
fn test_director_move_player_same_location() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_same_loc".to_string(), rules_json);

    let player_id = "test_player_same_loc";
    let mut player = Player::new(
        player_id.to_string(),
        "玩家E".to_string(),
        "password".to_string(),
        1,
        &game_state.rule_engine,
    );

    player.location = "位置1".to_string();
    game_state.players.insert(player_id.to_string(), player);
    game_state
        .places
        .insert("位置1".to_string(), Place::new("位置1".to_string()));

    // 导演尝试将玩家移动到当前位置
    let result = game_state.handle_move_player(player_id, "位置1");
    assert!(result.is_ok(), "移动到相同位置应该返回成功");

    let updated_player = game_state.players.get(player_id).unwrap();
    assert_eq!(updated_player.location, "位置1", "位置应该保持不变");
}

/// 测试：导演移动玩家到不存在的位置（应该失败）
#[test]
fn test_director_move_player_nonexistent_location() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_noexist_loc".to_string(), rules_json);

    let player_id = "test_player_noexist_loc";
    let mut player = Player::new(
        player_id.to_string(),
        "玩家F".to_string(),
        "password".to_string(),
        1,
        &game_state.rule_engine,
    );

    player.location = "位置1".to_string();
    game_state.players.insert(player_id.to_string(), player);
    game_state
        .places
        .insert("位置1".to_string(), Place::new("位置1".to_string()));

    // 导演尝试将玩家移动到不存在的位置
    let result = game_state.handle_move_player(player_id, "位置不存在");
    assert!(result.is_err(), "移动到不存在的位置应该失败");
    assert!(result.unwrap_err().contains("不存在"));
}

/// 测试：导演移动不存在的玩家（应该失败）
#[test]
fn test_director_move_nonexistent_player() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_noexist_player".to_string(), rules_json);

    game_state
        .places
        .insert("位置1".to_string(), Place::new("位置1".to_string()));
    game_state
        .places
        .insert("位置2".to_string(), Place::new("位置2".to_string()));

    // 导演尝试移动不存在的玩家
    let result = game_state.handle_move_player("nonexistent_player", "位置2");
    assert!(result.is_err(), "移动不存在的玩家应该失败");
    assert_eq!(result.unwrap_err(), "Player not found");
}

/// 测试：导演移动玩家到被摧毁的位置（应该返回失败提示但不移动）
#[test]
fn test_director_move_player_to_destroyed_location() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_destroyed".to_string(), rules_json);

    let player_id = "test_player_destroyed";
    let mut player = Player::new(
        player_id.to_string(),
        "玩家G".to_string(),
        "password".to_string(),
        1,
        &game_state.rule_engine,
    );

    player.location = "位置1".to_string();
    game_state.players.insert(player_id.to_string(), player);

    let location1 = Place::new("位置1".to_string());
    let mut location2 = Place::new("位置2".to_string());
    location2.is_destroyed = true;

    game_state.places.insert("位置1".to_string(), location1);
    game_state.places.insert("位置2".to_string(), location2);

    // 导演尝试将玩家移动到被摧毁的位置
    let result = game_state.handle_move_player(player_id, "位置2");
    assert!(
        result.is_ok(),
        "返回结果应该成功（包含失败信息），而不是返回Err"
    );

    // 验证玩家位置没有被改变（还在位置1）
    let player = game_state.players.get(player_id).unwrap();
    assert_eq!(
        player.location, "位置1",
        "被摧毁位置的移动失败，玩家应该留在原位置"
    );
}

/// 测试：货币类型信息验证
#[test]
fn test_currency_item_properties() {
    let rules_json =
        serde_json::to_string(&get_test_rules_with_currency()).expect("Failed to serialize rules");
    let rule_engine =
        GameRuleEngine::from_json(&rules_json).expect("Failed to parse test rules with currency");

    let gold_coin = rule_engine
        .create_item_from_name("金币")
        .expect("Failed to create gold coin");

    // 验证金币的所有属性
    assert_eq!(gold_coin.name, "金币");
    assert!(gold_coin.internal_name.is_none(), "没有设置internal_name");
    assert!(gold_coin.rarity.is_none(), "货币没有稀有度");

    // 验证物品类型和价值
    match &gold_coin.item_type {
        ItemType::Currency(props) => {
            assert_eq!(props.value, 100);
        }
        _ => panic!("应该是货币类型"),
    }
}

/// 测试：多个玩家独立的货币管理
#[test]
fn test_multiple_players_independent_coins() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_multi".to_string(), rules_json);

    // 创建两个玩家
    let player1_id = "player1";
    let player2_id = "player2";

    let mut player1 = Player::new(
        player1_id.to_string(),
        "玩家1".to_string(),
        "password1".to_string(),
        1,
        &game_state.rule_engine,
    );
    player1.location = "位置1".to_string();

    let mut player2 = Player::new(
        player2_id.to_string(),
        "玩家2".to_string(),
        "password2".to_string(),
        1,
        &game_state.rule_engine,
    );
    player2.location = "位置1".to_string();

    game_state.players.insert(player1_id.to_string(), player1);
    game_state.players.insert(player2_id.to_string(), player2);
    game_state
        .places
        .insert("位置1".to_string(), Place::new("位置1".to_string()));

    // 设置玩家1的货币为100
    game_state
        .handle_set_player_coins(player1_id, 100)
        .expect("Failed to set player1 coins");

    // 设置玩家2的货币为50
    game_state
        .handle_set_player_coins(player2_id, 50)
        .expect("Failed to set player2 coins");

    // 验证两个玩家的货币独立
    assert_eq!(game_state.players.get(player1_id).unwrap().coins, 100);
    assert_eq!(game_state.players.get(player2_id).unwrap().coins, 50);

    // 修改玩家1的货币不应该影响玩家2
    game_state
        .handle_set_player_coins(player1_id, 200)
        .expect("Failed to update player1 coins");

    assert_eq!(game_state.players.get(player1_id).unwrap().coins, 200);
    assert_eq!(game_state.players.get(player2_id).unwrap().coins, 50);
}

#[test]
fn test_shop_list_and_delist_item_broadcasts_to_all() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_shop_manage".to_string(), rules_json);

    add_test_player(&mut game_state, "player1", "玩家1", "位置1", 0);
    add_test_player(&mut game_state, "player2", "玩家2", "位置1", 0);

    let list_result = game_state
        .handle_shop_list_item("金币".to_string(), 15, 0)
        .expect("上架应成功");

    assert_eq!(game_state.shop.len(), 1);
    assert_eq!(game_state.shop[0].quantity, 1, "数量应自动至少为1");
    assert!(list_result.results[0].broadcast_to_all);
    assert_eq!(
        list_result.results[0].message_type,
        MessageType::SystemNotice
    );

    let listing_id = game_state.shop[0].id.clone();
    let delist_result = game_state
        .handle_shop_delist_item(&listing_id)
        .expect("下架应成功");

    assert!(game_state.shop.is_empty());
    assert!(delist_result.results[0].broadcast_to_all);
    assert_eq!(
        delist_result.results[0].message_type,
        MessageType::SystemNotice
    );
}

#[test]
fn test_shop_buy_success_splits_shop_sync_and_purchase_detail() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_shop_buy_success".to_string(), rules_json);

    add_test_player(&mut game_state, "buyer", "购买者", "位置1", 80);
    add_test_player(&mut game_state, "observer", "旁观者", "位置1", 0);
    game_state
        .shop
        .push(make_shop_listing("listing-1", "[HP10]测试药水", 25, 3));

    let results = game_state
        .handle_shop_buy_action(
            "buyer",
            &[ShopBuyItem {
                listing_id: "listing-1".to_string(),
                quantity: 2,
            }],
        )
        .expect("购买应成功");

    assert_eq!(
        results.results.len(),
        2,
        "应拆分为库存同步和购买明细两条结果"
    );

    let sync_result = &results.results[0];
    assert_eq!(sync_result.message_type, MessageType::Info);
    assert_eq!(sync_result.broadcast_players.len(), 2);
    assert!(sync_result.broadcast_players.iter().any(|id| id == "buyer"));
    assert!(
        sync_result
            .broadcast_players
            .iter()
            .any(|id| id == "observer")
    );
    assert!(sync_result.broadcast_to_director);
    assert!(!sync_result.broadcast_to_all);
    assert_eq!(sync_result.log_message, "商店库存已更新");

    let detail_result = &results.results[1];
    assert_eq!(detail_result.message_type, MessageType::SystemNotice);
    assert_eq!(detail_result.broadcast_players, vec!["buyer".to_string()]);
    assert!(detail_result.broadcast_to_director);
    assert!(!detail_result.broadcast_to_all);
    assert!(detail_result.log_message.contains("购买者"));
    assert!(detail_result.log_message.contains("花费 50 货币"));

    let buyer = game_state.players.get("buyer").unwrap();
    assert_eq!(buyer.coins, 30);
    assert_eq!(buyer.inventory.len(), 2);
    assert_eq!(game_state.shop.len(), 1);
    assert_eq!(game_state.shop[0].quantity, 1);
}

#[test]
fn test_shop_buy_rejects_duplicate_listing_total_exceeding_stock() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_shop_buy_duplicate".to_string(), rules_json);

    add_test_player(&mut game_state, "buyer", "购买者", "位置1", 100);
    game_state
        .shop
        .push(make_shop_listing("listing-1", "测试药水", 10, 1));

    let result = game_state
        .handle_shop_buy_action(
            "buyer",
            &[
                ShopBuyItem {
                    listing_id: "listing-1".to_string(),
                    quantity: 1,
                },
                ShopBuyItem {
                    listing_id: "listing-1".to_string(),
                    quantity: 1,
                },
            ],
        )
        .expect("重复购买应返回提示结果而不是Err");

    assert_eq!(result.results.len(), 1);
    assert_eq!(result.results[0].message_type, MessageType::Info);
    assert!(result.results[0].log_message.contains("库存不足"));
    assert_eq!(game_state.players.get("buyer").unwrap().coins, 100);
    assert!(
        game_state
            .players
            .get("buyer")
            .unwrap()
            .inventory
            .is_empty()
    );
    assert_eq!(game_state.shop[0].quantity, 1);
}

#[test]
fn test_shop_buy_rejects_when_total_cost_multiplication_overflows() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_shop_buy_mul_overflow".to_string(), rules_json);

    add_test_player(&mut game_state, "buyer", "购买者", "位置1", i32::MAX);
    game_state
        .shop
        .push(make_shop_listing("listing-1", "测试药水", i32::MAX, 2));

    let result = game_state
        .handle_shop_buy_action(
            "buyer",
            &[ShopBuyItem {
                listing_id: "listing-1".to_string(),
                quantity: 2,
            }],
        )
        .expect("溢出应返回提示结果而不是Err");

    assert_eq!(result.results[0].message_type, MessageType::Info);
    assert!(result.results[0].log_message.contains("总价计算溢出"));
    assert_eq!(game_state.players.get("buyer").unwrap().coins, i32::MAX);
    assert!(
        game_state
            .players
            .get("buyer")
            .unwrap()
            .inventory
            .is_empty()
    );
    assert_eq!(game_state.shop[0].quantity, 2);
}

#[test]
fn test_shop_buy_rejects_when_total_cost_accumulation_overflows() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_shop_buy_add_overflow".to_string(), rules_json);

    add_test_player(&mut game_state, "buyer", "购买者", "位置1", i32::MAX);
    game_state
        .shop
        .push(make_shop_listing("listing-1", "测试药水", i32::MAX, 1));
    game_state
        .shop
        .push(make_shop_listing("listing-2", "金币", i32::MAX, 1));

    let result = game_state
        .handle_shop_buy_action(
            "buyer",
            &[
                ShopBuyItem {
                    listing_id: "listing-1".to_string(),
                    quantity: 1,
                },
                ShopBuyItem {
                    listing_id: "listing-2".to_string(),
                    quantity: 1,
                },
            ],
        )
        .expect("溢出应返回提示结果而不是Err");

    assert_eq!(result.results[0].message_type, MessageType::Info);
    assert!(result.results[0].log_message.contains("总价过大"));
    assert_eq!(game_state.players.get("buyer").unwrap().coins, i32::MAX);
    assert!(
        game_state
            .players
            .get("buyer")
            .unwrap()
            .inventory
            .is_empty()
    );
    assert_eq!(game_state.shop[0].quantity, 1);
    assert_eq!(game_state.shop[1].quantity, 1);
}

#[test]
fn test_shop_buy_rolls_back_when_item_creation_fails() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_shop_buy_atomic".to_string(), rules_json);

    add_test_player(&mut game_state, "buyer", "购买者", "位置1", 100);
    game_state
        .shop
        .push(make_shop_listing("listing-1", "不存在的商品", 10, 1));

    let result = game_state
        .handle_shop_buy_action(
            "buyer",
            &[ShopBuyItem {
                listing_id: "listing-1".to_string(),
                quantity: 1,
            }],
        )
        .expect("创建失败应返回提示结果而不是Err");

    assert_eq!(result.results[0].message_type, MessageType::Info);
    assert!(
        result.results[0]
            .log_message
            .contains("创建物品 不存在的商品 失败")
    );
    assert_eq!(game_state.players.get("buyer").unwrap().coins, 100);
    assert!(
        game_state
            .players
            .get("buyer")
            .unwrap()
            .inventory
            .is_empty()
    );
    assert_eq!(game_state.shop[0].quantity, 1);
}

#[test]
fn test_shop_buy_rejects_insufficient_balance_without_mutation() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_shop_buy_balance".to_string(), rules_json);

    add_test_player(&mut game_state, "buyer", "购买者", "位置1", 20);
    game_state
        .shop
        .push(make_shop_listing("listing-1", "测试药水", 30, 1));

    let result = game_state
        .handle_shop_buy_action(
            "buyer",
            &[ShopBuyItem {
                listing_id: "listing-1".to_string(),
                quantity: 1,
            }],
        )
        .expect("余额不足应返回提示结果而不是Err");

    assert_eq!(result.results[0].message_type, MessageType::Info);
    assert!(result.results[0].log_message.contains("货币不足"));
    assert_eq!(game_state.players.get("buyer").unwrap().coins, 20);
    assert!(
        game_state
            .players
            .get("buyer")
            .unwrap()
            .inventory
            .is_empty()
    );
    assert_eq!(game_state.shop[0].quantity, 1);
}

#[test]
fn test_currency_item_use_supports_boundary_value() {
    let rules_json = get_test_rules_with_single_currency("超大金币", i32::MAX);
    let mut game_state = GameState::new("test_game_currency_boundary".to_string(), rules_json);

    add_test_player(&mut game_state, "player", "玩家", "位置1", 0);
    let item = game_state
        .rule_engine
        .create_item_from_name("超大金币")
        .expect("应能创建边界货币道具");
    game_state
        .players
        .get_mut("player")
        .unwrap()
        .inventory
        .push(item);

    let item_id = game_state.players["player"].inventory[0].id.clone();
    let result = game_state
        .handle_use_action("player", &item_id, &empty_action_params())
        .expect("使用边界货币应成功");

    assert_eq!(result.results.len(), 1);
    assert_eq!(result.results[0].message_type, MessageType::SystemNotice);
    assert_eq!(game_state.players.get("player").unwrap().coins, i32::MAX);
    assert!(
        game_state
            .players
            .get("player")
            .unwrap()
            .inventory
            .is_empty()
    );
}

#[test]
fn test_currency_item_use_rejects_overflow_and_keeps_item() {
    let rules_json = get_test_rules_with_single_currency("超大金币", i32::MAX);
    let mut game_state = GameState::new("test_game_currency_overflow".to_string(), rules_json);

    add_test_player(&mut game_state, "player", "玩家", "位置1", 1);
    let item = game_state
        .rule_engine
        .create_item_from_name("超大金币")
        .expect("应能创建边界货币道具");
    game_state
        .players
        .get_mut("player")
        .unwrap()
        .inventory
        .push(item);

    let item_id = game_state.players["player"].inventory[0].id.clone();
    let result = game_state
        .handle_use_action("player", &item_id, &empty_action_params())
        .expect("溢出应返回提示结果而不是Err");

    assert_eq!(result.results[0].message_type, MessageType::Info);
    assert!(result.results[0].log_message.contains("货币总数溢出"));
    assert_eq!(game_state.players.get("player").unwrap().coins, 1);
    assert_eq!(game_state.players.get("player").unwrap().inventory.len(), 1);
}

#[test]
fn test_kill_player_transfers_coins_for_player_kill() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_kill_transfer".to_string(), rules_json);

    add_test_player(&mut game_state, "killer", "击杀者", "位置1", 10);
    add_test_player(&mut game_state, "victim", "受害者", "位置1", 25);

    let result = game_state
        .kill_player("victim", Some("killer"), Some("killer"), "攻击")
        .expect("击杀应成功");

    assert_eq!(game_state.players.get("killer").unwrap().coins, 35);
    assert_eq!(game_state.players.get("victim").unwrap().coins, 0);
    assert_eq!(result.results[0].data["transferred_coins"], json!(25));
}

#[test]
fn test_kill_player_transfers_coins_for_bleed_death() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_bleed_transfer".to_string(), rules_json);

    add_test_player(&mut game_state, "killer", "击杀者", "位置1", 5);
    add_test_player(&mut game_state, "victim", "受害者", "位置1", 20);

    let result = game_state
        .kill_player("victim", Some("killer"), Some("killer"), "流血")
        .expect("流血致死应成功");

    assert_eq!(game_state.players.get("killer").unwrap().coins, 25);
    assert_eq!(result.results[0].data["reason"], json!("流血"));
    assert_eq!(result.results[0].data["transferred_coins"], json!(20));
}

#[test]
fn test_kill_player_drops_coins_on_shrink_death() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_shrink_death".to_string(), rules_json);

    add_test_player(&mut game_state, "victim", "受害者", "位置1", 18);

    let result = game_state
        .kill_player("victim", None, None, "缩圈")
        .expect("缩圈致死应成功");

    assert_eq!(game_state.players.get("victim").unwrap().coins, 0);
    assert_eq!(result.results[0].data["transferred_coins"], json!(0));
    assert!(result.results[0].log_message.contains("消失货币: 18"));
}

#[test]
fn test_kill_player_drops_coins_on_director_kill() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_director_kill".to_string(), rules_json);

    add_test_player(&mut game_state, "victim", "受害者", "位置1", 12);

    let result = game_state
        .kill_player("victim", None, None, "导演处决")
        .expect("导演击杀应成功");

    assert_eq!(game_state.players.get("victim").unwrap().coins, 0);
    assert_eq!(result.results[0].data["transferred_coins"], json!(0));
    assert!(result.results[0].log_message.contains("消失货币: 12"));
}

#[test]
fn test_kill_player_coin_transfer_overflow_does_not_wrap() {
    let rules_json = get_test_rules_with_currency();
    let mut game_state = GameState::new("test_game_kill_overflow".to_string(), rules_json);

    add_test_player(&mut game_state, "killer", "击杀者", "位置1", i32::MAX);
    add_test_player(&mut game_state, "victim", "受害者", "位置1", 1);

    let result = game_state
        .kill_player("victim", Some("killer"), Some("killer"), "攻击")
        .expect("击杀应成功");

    assert_eq!(game_state.players.get("killer").unwrap().coins, i32::MAX);
    assert_eq!(game_state.players.get("victim").unwrap().coins, 0);
    assert_eq!(result.results[0].data["transferred_coins"], json!(0));
    assert!(result.results[0].log_message.contains("消失货币: 1"));
}

#[test]
fn test_legacy_save_deserialization_defaults_for_coins_shop_and_quantity() {
    let rules_json = get_test_rules_with_currency();

    let mut player_value = serde_json::to_value(Player::new(
        "legacy_player".to_string(),
        "旧玩家".to_string(),
        "password".to_string(),
        1,
        &GameRuleEngine::from_json(&serde_json::to_string(&rules_json).expect("规则序列化应成功"))
            .expect("规则解析应成功"),
    ))
    .expect("玩家序列化应成功");
    player_value.as_object_mut().unwrap().remove("coins");

    let game_state_without_shop: GameState = serde_json::from_value(json!({
        "game_id": "legacy-game-1",
        "players": {
            "legacy_player": player_value.clone()
        },
        "places": {},
        "weather": 1.0,
        "votes": {},
        "rules_config": rules_json.clone(),
        "night_start_time": null,
        "night_end_time": null,
        "next_night_destroyed_places": [],
        "save_time": null
    }))
    .expect("旧存档应可反序列化");

    assert!(game_state_without_shop.shop.is_empty());
    assert_eq!(game_state_without_shop.players["legacy_player"].coins, 0);

    let game_state_with_legacy_listing: GameState = serde_json::from_value(json!({
        "game_id": "legacy-game-2",
        "players": {
            "legacy_player": player_value
        },
        "places": {},
        "weather": 1.0,
        "votes": {},
        "rules_config": rules_json,
        "night_start_time": null,
        "night_end_time": null,
        "next_night_destroyed_places": [],
        "save_time": null,
        "shop": [
            {
                "id": "listing-1",
                "item_name": "金币",
                "price": 10
            }
        ]
    }))
    .expect("旧商店存档应可反序列化");

    assert_eq!(game_state_with_legacy_listing.shop.len(), 1);
    assert_eq!(game_state_with_legacy_listing.shop[0].quantity, 1);
    assert_eq!(
        game_state_with_legacy_listing.players["legacy_player"].coins,
        0
    );
}
