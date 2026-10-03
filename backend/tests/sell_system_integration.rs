//! 售出系统集成测试

use chrono::{Duration, Utc};
use royale_arena_backend::game::game_rule_engine::GameRuleEngine;
use royale_arena_backend::websocket::models::GameState;
use serde_json::{Value, json};

#[test]
fn sell_prices_reject_nonfinite_oversized_and_fractional_inputs() {
    let mut state = build_sell_game_state();
    for price in [1e308, f64::INFINITY, f64::NAN, 9999.5, 0.6, 0.0, -0.5] {
        let results = state.handle_sell_set_price("common".into(), price).unwrap();
        assert_eq!(results.results[0].message_type, MessageType::Info);
        assert!(state.sell_prices.is_empty());
    }
    state
        .handle_sell_set_price("common".into(), 9999.0)
        .unwrap();
    assert_eq!(state.sell_prices[0].price, 9999.0);
    state.handle_sell_set_price("common".into(), 1e308).unwrap();
    assert_eq!(state.sell_prices[0].price, 9999.0);
}

#[test]
fn sale_rejects_bad_saved_price_and_balance_overflow_atomically() {
    use royale_arena_backend::websocket::currency::MAX_COINS;
    for (balance, price) in [(0.0, 1e308), (0.0, f64::NAN), (MAX_COINS, 0.5)] {
        let mut state = build_sell_game_state();
        sell_add_player(&mut state, "p1", "玩家一");
        state.players.get_mut("p1").unwrap().coins = balance;
        sell_configure(&mut state, "common", 0.5);
        state.sell_prices[0].price = price;
        sell_put_weapon(&mut state, "p1", "w", Some("common"));
        sell_put_armor(&mut state, "p1", "a", Some("common"));
        sell_set_night_window(&mut state, 3600, 7200);
        assert!(
            state
                .handle_sell_item_action("p1", &["w".into(), "a".into()])
                .is_err()
        );
        assert_eq!(state.players["p1"].coins, balance);
        assert_eq!(state.players["p1"].inventory.len(), 2);
    }
}

const SELL_RULES: &str = r#"{
    "map": { "places": ["码头"], "safe_places": [] },
    "player": { "max_life": 100, "max_strength": 100, "daily_life_recovery": 0,
                 "daily_strength_recovery": 40, "search_cooldown": 30,
                 "max_backpack_items": 6, "unarmed_damage": 5 },
    "action_costs": { "move": 5, "search": 5, "pick": 0, "attack": 0,
                       "equip": 0, "use": 0, "throw": 0, "deliver": 10 },
    "rest_mode": { "life_recovery": 25, "strength_recovery": 1000, "max_moves": 1 },
    "teammate_behavior": 0,
    "death_item_disposition": "killer_takes_loot",
    "display_names": {},
    "items_config": { "rarity_levels": [], "items": {}, "upgrade_recipes": {} }
}"#;

pub fn sell_test_rules(_mode: i32) -> Value {
    serde_json::from_str(SELL_RULES).unwrap()
}

fn build_sell_game_state() -> GameState {
    GameState::new("sell_game".to_string(), sell_test_rules(0))
}

fn sell_add_player(state: &mut GameState, id: &str, name: &str) {
    let engine = GameRuleEngine::from_json(&state.rules_config.to_string()).unwrap();
    let player = royale_arena_backend::websocket::models::Player::new(
        id.to_string(),
        name.to_string(),
        "pw".to_string(),
        0,
        &engine,
    );
    state.players.insert(id.to_string(), player);
}

#[allow(dead_code)] // 后续售出任务复用
fn sell_set_night_window(state: &mut GameState, start_offset_secs: i64, end_offset_secs: i64) {
    state.night_start_time = Some(Utc::now() + Duration::seconds(start_offset_secs));
    state.night_end_time = Some(Utc::now() + Duration::seconds(end_offset_secs));
}

#[test]
fn coins_supports_fractional_values() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    let p = state.players.get_mut("p1").unwrap();
    p.coins += 0.5;
    p.coins += 0.5;
    let p = state.players.get("p1").unwrap();
    assert!(
        (p.coins - 1.0).abs() < 1e-9,
        "两件 0.5 应累计为 1.0，实际 {}",
        p.coins
    );
}

#[test]
fn game_state_deserializes_without_sell_prices() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    let serialized = serde_json::to_string(&state).unwrap();
    // 模拟旧存档：手工删掉 sell_prices 字段后仍能加载
    let legacy = serialized
        .replace("\"sell_prices\":[]", "")
        .replace(",}", "}");
    let restored: GameState =
        serde_json::from_str(&legacy).expect("旧存档（无 sell_prices）必须能反序列化");
    assert!(restored.sell_prices.is_empty());
}

use royale_arena_backend::game::models::MessageType;
use royale_arena_backend::websocket::actions::director_action_scheduler::{
    DirectorActionParams, DirectorActionScheduler,
};

fn director_dispatch(
    state: &mut GameState,
    params: serde_json::Value,
) -> Result<royale_arena_backend::websocket::models::ActionResults, String> {
    let action_type = params["action_type"].as_str().unwrap_or("").to_string();
    let ap = DirectorActionParams::from_json(&params)?;
    DirectorActionScheduler::dispatch(state, &action_type, ap)
}

#[test]
fn sell_set_price_adds_entry() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    let results = director_dispatch(
        &mut state,
        serde_json::json!({"action_type": "sell_set_price", "sell_rarity": "common", "sell_price": 2.5}),
    )
    .expect("dispatch ok");
    assert_eq!(results.results[0].message_type, MessageType::SystemNotice);
    assert_eq!(state.sell_prices.len(), 1);
    assert_eq!(state.sell_prices[0].rarity, "common");
    assert!((state.sell_prices[0].price - 2.5).abs() < 1e-9);
}

#[test]
fn sell_set_price_updates_existing_entry() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    director_dispatch(
        &mut state,
        serde_json::json!({"action_type":"sell_set_price","sell_rarity":"rare","sell_price":1.0}),
    )
    .unwrap();
    director_dispatch(
        &mut state,
        serde_json::json!({"action_type":"sell_set_price","sell_rarity":"rare","sell_price":3.0}),
    )
    .unwrap();
    assert_eq!(state.sell_prices.len(), 1, "同一稀有度只有一条");
    assert!((state.sell_prices[0].price - 3.0).abs() < 1e-9, "改价生效");
}

#[test]
fn sell_set_price_rejects_bad_rarity_and_price() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    for (rarity, price) in [
        ("purple", 1.0),
        ("common", 0.0),
        ("common", -0.5),
        ("common", 0.3),
    ] {
        let results = director_dispatch(
            &mut state,
            serde_json::json!({"action_type":"sell_set_price","sell_rarity":rarity,"sell_price":price}),
        )
        .expect("ok");
        assert_eq!(
            results.results[0].message_type,
            MessageType::Info,
            "{}@{} 应被拒绝",
            rarity,
            price
        );
    }
    assert!(state.sell_prices.is_empty(), "拒绝时不产生条目");
}

#[test]
fn sell_remove_price_works() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    director_dispatch(
        &mut state,
        serde_json::json!({"action_type":"sell_set_price","sell_rarity":"epic","sell_price":4.0}),
    )
    .unwrap();
    let results = director_dispatch(
        &mut state,
        serde_json::json!({"action_type":"sell_remove_price","sell_rarity":"epic"}),
    )
    .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::SystemNotice);
    assert!(state.sell_prices.is_empty());
    // 删除不存在的稀有度 → Info
    let results = director_dispatch(
        &mut state,
        serde_json::json!({"action_type":"sell_remove_price","sell_rarity":"epic"}),
    )
    .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
}

use royale_arena_backend::game::game_rule_engine::{
    ArmorProperties, CurrencyProperties, Item, ItemType, WeaponProperties,
};

fn sell_put_weapon(state: &mut GameState, player_id: &str, item_id: &str, rarity: Option<&str>) {
    let item = Item {
        id: item_id.to_string(),
        name: format!("[W]{}", item_id),
        internal_name: Some("test_weapon".to_string()),
        rarity: rarity.map(|r| r.to_string()),
        item_type: ItemType::Weapon(WeaponProperties {
            damage: 5,
            votes: 1,
            uses: None,
            aoe_damage: None,
            bleed_damage: None,
        }),
    };
    state
        .players
        .get_mut(player_id)
        .unwrap()
        .inventory
        .push(item);
}

fn sell_put_armor(state: &mut GameState, player_id: &str, item_id: &str, rarity: Option<&str>) {
    let item = Item {
        id: item_id.to_string(),
        name: format!("[A]{}", item_id),
        internal_name: Some("test_armor".to_string()),
        rarity: rarity.map(|r| r.to_string()),
        item_type: ItemType::Armor(ArmorProperties {
            defense: 5,
            votes: 1,
            uses: None,
        }),
    };
    state
        .players
        .get_mut(player_id)
        .unwrap()
        .inventory
        .push(item);
}

fn sell_configure(state: &mut GameState, rarity: &str, price: f64) {
    state
        .handle_sell_set_price(rarity.to_string(), price)
        .unwrap();
}

#[test]
fn sell_item_succeeds_in_daytime() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "rare", 2.5);
    sell_put_weapon(&mut state, "p1", "w1", Some("rare"));
    // 夜窗设在未来 → 当前是白天
    sell_set_night_window(&mut state, 3600, 7200);

    let results = state
        .handle_sell_item_action("p1", &["w1".to_string()])
        .expect("sell ok");
    let p = state.players.get("p1").unwrap();
    assert!(p.inventory.iter().all(|i| i.id != "w1"), "道具已移除");
    assert!((p.coins - 2.5).abs() < 1e-9, "货币 +2.5，实际 {}", p.coins);
    assert_eq!(results.results.len(), 2, "发起方 + 导演专属");
    assert_eq!(results.results[0].message_type, MessageType::SystemNotice);
    assert_eq!(results.results[0].broadcast_players, vec!["p1".to_string()]);
    assert!(!results.results[0].broadcast_to_director);
    let director_msg = &results.results[1];
    assert!(director_msg.broadcast_players.is_empty());
    assert!(director_msg.broadcast_to_director);
    assert!(director_msg.log_message.contains("玩家一"));
    assert!(director_msg.log_message.contains("蓝"));
    assert!(director_msg.log_message.contains("2.5"));
}

#[test]
fn sell_item_rejected_at_night() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.0);
    sell_put_weapon(&mut state, "p1", "w1", Some("common"));
    sell_set_night_window(&mut state, -3600, 3600); // 当前在夜内
    let results = state
        .handle_sell_item_action("p1", &["w1".to_string()])
        .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(results.results[0].log_message.contains("非夜间"));
    assert_eq!(state.players["p1"].inventory.len(), 1, "道具保留");
}

#[test]
fn sell_item_rejected_when_night_unset() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.0);
    sell_put_weapon(&mut state, "p1", "w1", Some("common"));
    let results = state
        .handle_sell_item_action("p1", &["w1".to_string()])
        .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(results.results[0].log_message.contains("尚未设置夜晚"));
}

#[test]
fn sell_item_rejects_non_weapon_armor_and_unconfigured_rarity_and_missing_item() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.0);
    sell_set_night_window(&mut state, 3600, 7200);
    // 未配置稀有度
    sell_put_armor(&mut state, "p1", "a1", Some("epic"));
    let r = state
        .handle_sell_item_action("p1", &["a1".to_string()])
        .unwrap();
    assert_eq!(r.results[0].message_type, MessageType::Info);
    assert!(r.results[0].log_message.contains("未开放售出"));
    // rarity 为 None
    sell_put_weapon(&mut state, "p1", "w2", None);
    let r = state
        .handle_sell_item_action("p1", &["w2".to_string()])
        .unwrap();
    assert_eq!(r.results[0].message_type, MessageType::Info);
    assert!(r.results[0].log_message.contains("未开放售出"));
    // 不在背包
    let r = state
        .handle_sell_item_action("p1", &["nope".to_string()])
        .unwrap();
    assert_eq!(r.results[0].message_type, MessageType::Info);
    assert!(r.results[0].log_message.contains("不在背包"));
    // 仅存在于 equipped_weapon（不在 inventory）的道具不可售
    let mut w = state.players.get("p1").unwrap().inventory[1].clone();
    w.id = "equipped_only".to_string();
    state.players.get_mut("p1").unwrap().equipped_weapon = Some(w);
    let r = state
        .handle_sell_item_action("p1", &["equipped_only".to_string()])
        .unwrap();
    assert_eq!(r.results[0].message_type, MessageType::Info);
    assert!(r.results[0].log_message.contains("不在背包"));
    assert_eq!(state.players["p1"].inventory.len(), 2, "无状态变更");
}

#[test]
fn sell_item_rejects_non_weapon_types() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.0);
    // 用货币道具（ItemType::Currency）验证非武器/防具被拒
    let item = Item {
        id: "c1".to_string(),
        name: "金币".to_string(),
        internal_name: Some("gold_coin".to_string()),
        rarity: Some("common".to_string()),
        item_type: ItemType::Currency(CurrencyProperties { value: 1 }),
    };
    state.players.get_mut("p1").unwrap().inventory.push(item);
    sell_set_night_window(&mut state, 3600, 7200);
    let r = state
        .handle_sell_item_action("p1", &["c1".to_string()])
        .unwrap();
    assert_eq!(r.results[0].message_type, MessageType::Info);
    assert!(r.results[0].log_message.contains("武器和防具"));
}

#[test]
fn sell_item_via_scheduler_dispatch() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    state.players.get_mut("p1").unwrap().location = "码头".to_string(); // 通过 Born 校验
    sell_configure(&mut state, "rare", 0.5);
    sell_put_weapon(&mut state, "p1", "w1", Some("rare"));
    sell_set_night_window(&mut state, 3600, 7200);
    use royale_arena_backend::websocket::actions::player_action_scheduler::{
        ActionParams, PlayerActionScheduler,
    };
    let params = ActionParams::from_json(&json!({ "item_ids": ["w1"] })).unwrap();
    let results = PlayerActionScheduler::dispatch(&mut state, "p1", "sell_item", params)
        .expect("dispatch ok");
    assert!((state.players["p1"].coins - 0.5).abs() < 1e-9);
    assert_eq!(results.results.len(), 2);
}

#[test]
fn client_json_includes_sell_prices() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.5);
    let json = state.to_player_client_json();
    let arr = json["sell_prices"]
        .as_array()
        .expect("sell_prices 必须存在");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["rarity"], "common");
    assert!((arr[0]["price"].as_f64().unwrap() - 1.5).abs() < 1e-9);
    let djson = state.to_director_client_json();
    assert!(djson["sell_prices"].as_array().unwrap().len() == 1);
}

#[test]
fn sell_green_pair_succeeds() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.5);
    sell_put_weapon(&mut state, "p1", "w1", Some("common"));
    sell_put_armor(&mut state, "p1", "a1", Some("common"));
    sell_set_night_window(&mut state, 3600, 7200);

    let results = state
        .handle_sell_item_action("p1", &["w1".to_string(), "a1".to_string()])
        .expect("sell ok");
    let p = state.players.get("p1").unwrap();
    assert!(p.inventory.is_empty(), "两件都已移除");
    assert!((p.coins - 3.0).abs() < 1e-9, "1.5 × 2，实际 {}", p.coins);
    assert_eq!(results.results.len(), 2);
    let expected_names = json!(["[W]w1", "[A]a1"]);
    assert_eq!(results.results[0].data["item_names"], expected_names);
    assert_eq!(results.results[1].data["item_names"], expected_names);
    assert_eq!(
        results.results[1].data["items"],
        json!([
            {"name": "[W]w1", "rarity": "common"},
            {"name": "[A]a1", "rarity": "common"},
        ])
    );
    let seller_msg = &results.results[0].log_message;
    assert!(seller_msg.contains("、"), "名字用顿号连接: {}", seller_msg);
    assert!(seller_msg.contains("[W]w1") && seller_msg.contains("[A]a1"));
    let director_msg = &results.results[1].log_message;
    assert!(director_msg.contains("绿"));
    assert!(director_msg.contains("3"));
}

#[test]
fn sell_single_green_rejected() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.5);
    sell_put_weapon(&mut state, "p1", "w1", Some("common"));
    sell_set_night_window(&mut state, 3600, 7200);

    let results = state
        .handle_sell_item_action("p1", &["w1".to_string()])
        .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        results.results[0]
            .log_message
            .contains("绿色物品需成对售出")
    );
    let p = state.players.get("p1").unwrap();
    assert_eq!(p.inventory.len(), 1, "零状态变更");
    assert!((p.coins - 0.0).abs() < 1e-9);
}

#[test]
fn sell_green_mixed_with_rare_rejected() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.5);
    sell_configure(&mut state, "rare", 3.0);
    sell_put_weapon(&mut state, "p1", "w1", Some("common"));
    sell_put_weapon(&mut state, "p1", "w2", Some("rare"));
    sell_set_night_window(&mut state, 3600, 7200);

    let results = state
        .handle_sell_item_action("p1", &["w1".to_string(), "w2".to_string()])
        .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        results.results[0]
            .log_message
            .contains("绿色物品不能与其他稀有度混合售出")
    );
    let p = state.players.get("p1").unwrap();
    assert_eq!(p.inventory.len(), 2, "零状态变更");
    assert!((p.coins - 0.0).abs() < 1e-9);
}

#[test]
fn sell_two_rare_rejected() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "rare", 3.0);
    sell_put_weapon(&mut state, "p1", "w1", Some("rare"));
    sell_put_weapon(&mut state, "p1", "w2", Some("rare"));
    sell_set_night_window(&mut state, 3600, 7200);

    let results = state
        .handle_sell_item_action("p1", &["w1".to_string(), "w2".to_string()])
        .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        results.results[0]
            .log_message
            .contains("绿色物品不能与其他稀有度混合售出")
    );
    assert_eq!(state.players.get("p1").unwrap().inventory.len(), 2);
}

#[test]
fn sell_invalid_length_rejected() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.5);
    for id in ["w1", "w2", "w3"] {
        sell_put_weapon(&mut state, "p1", id, Some("common"));
    }
    sell_set_night_window(&mut state, 3600, 7200);

    let three = state
        .handle_sell_item_action(
            "p1",
            &["w1".to_string(), "w2".to_string(), "w3".to_string()],
        )
        .unwrap();
    assert_eq!(three.results[0].message_type, MessageType::Info);
    assert!(
        three.results[0]
            .log_message
            .contains("一次只能售出 1 件非绿色物品或 2 件绿色物品")
    );
    let empty = state.handle_sell_item_action("p1", &[]).unwrap();
    assert_eq!(empty.results[0].message_type, MessageType::Info);
    assert!(
        empty.results[0]
            .log_message
            .contains("一次只能售出 1 件非绿色物品或 2 件绿色物品")
    );
    assert_eq!(state.players.get("p1").unwrap().inventory.len(), 3);
}

#[test]
fn sell_pair_with_missing_item_rejected() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.5);
    sell_put_weapon(&mut state, "p1", "w1", Some("common"));
    sell_set_night_window(&mut state, 3600, 7200);

    let results = state
        .handle_sell_item_action("p1", &["w1".to_string(), "nope".to_string()])
        .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(results.results[0].log_message.contains("不在背包中"));
    assert_eq!(
        state.players.get("p1").unwrap().inventory.len(),
        1,
        "零状态变更"
    );
    assert!((state.players.get("p1").unwrap().coins - 0.0).abs() < 1e-9);
}

#[test]
fn sell_duplicate_item_id_rejected() {
    let mut state = build_sell_game_state();
    sell_add_player(&mut state, "p1", "玩家一");
    sell_configure(&mut state, "common", 1.5);
    sell_put_weapon(&mut state, "p1", "w1", Some("common"));
    sell_set_night_window(&mut state, 3600, 7200);

    let results = state
        .handle_sell_item_action("p1", &["w1".to_string(), "w1".to_string()])
        .unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        results.results[0]
            .log_message
            .contains("不能重复选择同一件物品")
    );
    let p = state.players.get("p1").unwrap();
    assert_eq!(p.inventory.len(), 1, "零状态变更");
    assert!((p.coins - 0.0).abs() < 1e-9);
}
