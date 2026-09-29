//! 商店稀有度类目上架集成测试

use royale_arena_backend::game::game_rule_engine::GameRuleEngine;
use royale_arena_backend::game::models::MessageType;
use royale_arena_backend::websocket::actions::director_action_scheduler::{
    DirectorActionParams, DirectorActionScheduler,
};
use royale_arena_backend::websocket::models::{GameState, ShopListing};
use serde_json::{json, Value};

const SHOP_RARITY_RULES: &str = r#"{
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
    "items_config": {
        "rarity_levels": [],
        "items": {
            "weapons": [
                { "internal_name": "test_sword", "display_names": ["青钢剑", "铁刃短剑"],
                  "rarity": "common", "properties": { "damage": 20, "votes": 1 } },
                { "internal_name": "test_rare_blade", "display_names": ["秘银长刃"],
                  "rarity": "rare", "properties": { "damage": 30, "votes": 1 } }
            ],
            "armors": [
                { "internal_name": "test_leather", "display_names": ["旧皮甲", "硬革胸甲"],
                  "rarity": "common", "properties": { "defense": 5, "votes": 1 } }
            ],
            "utilities": [], "upgraders": [], "currencies": [],
            "consumables": [
                { "name": "[HP10]测试药水",
                  "properties": { "effect_type": "heal", "effect_value": 10 } }
            ]
        },
        "upgrade_recipes": {}
    }
}"#;

pub fn shop_rarity_test_rules(_mode: i32) -> Value {
    serde_json::from_str(SHOP_RARITY_RULES).unwrap()
}

fn build_shop_rarity_state() -> GameState {
    GameState::new("shop_rarity_game".to_string(), shop_rarity_test_rules(0))
}

fn shop_add_player(state: &mut GameState, id: &str, name: &str) {
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

fn director_dispatch(
    state: &mut GameState,
    params: serde_json::Value,
) -> Result<royale_arena_backend::websocket::models::ActionResults, String> {
    let action_type = params["action_type"].as_str().unwrap_or("").to_string();
    let ap = DirectorActionParams::from_json(&params)?;
    DirectorActionScheduler::dispatch(state, &action_type, ap)
}

#[test]
fn shop_list_rarity_succeeds() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    let results = director_dispatch(&mut state, json!({
        "action_type": "shop_list_rarity",
        "shop_item_kind": "weapon", "shop_rarity": "common",
        "price": 2, "quantity": 2
    })).expect("dispatch ok");
    assert_eq!(results.results[0].message_type, MessageType::SystemNotice);
    assert_eq!(state.shop.len(), 1);
    let l = &state.shop[0];
    assert_eq!(l.item_kind.as_deref(), Some("weapon"));
    assert_eq!(l.rarity.as_deref(), Some("common"));
    assert_eq!(l.item_name, "绿类武器（随机）");
    assert_eq!(l.price, 2);
    assert_eq!(l.quantity, 2);
}

#[test]
fn shop_list_rarity_rejects_invalid_inputs() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    let cases = vec![
        // (kind, rarity, price, quantity, 期望消息片段)
        ("shield", "common", 1, 1, "weapon 或 armor"),
        ("weapon", "purple", 1, 1, "common/rare/epic/legendary"),
        ("weapon", "common", 0, 1, "上架价格必须 >= 1"),
        ("weapon", "legendary", 1, 1, "道具库中没有橙类武器"),
        ("armor", "legendary", 1, 1, "道具库中没有橙类防具"),
        ("weapon", "common", 1, 3, "名称总数"),      // common 武器池只有 2 个名字
        ("armor", "common", 1, 5, "名称总数"),        // common 防具池只有 2 个名字
    ];
    for (kind, rarity, price, quantity, fragment) in cases {
        let results = director_dispatch(&mut state, json!({
            "action_type": "shop_list_rarity",
            "shop_item_kind": kind, "shop_rarity": rarity,
            "price": price, "quantity": quantity
        })).expect("dispatch ok");
        assert_eq!(results.results[0].message_type, MessageType::Info,
            "{}@{}@{}@{} 应被拒绝", kind, rarity, price, quantity);
        assert!(results.results[0].log_message.contains(fragment),
            "消息 {:?} 应包含 {:?}", results.results[0].log_message, fragment);
    }
    assert!(state.shop.is_empty(), "拒绝时不产生条目");
}

#[test]
fn shop_list_rarity_rejects_duplicate_category() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    director_dispatch(&mut state, json!({
        "action_type": "shop_list_rarity",
        "shop_item_kind": "weapon", "shop_rarity": "common", "price": 2, "quantity": 1
    })).unwrap();
    let results = director_dispatch(&mut state, json!({
        "action_type": "shop_list_rarity",
        "shop_item_kind": "weapon", "shop_rarity": "common", "price": 3, "quantity": 1
    })).unwrap();
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(results.results[0].log_message.contains("该类目已上架"));
    assert_eq!(state.shop.len(), 1, "同稀有度只有一条");
    // 不同稀有度可再上架
    director_dispatch(&mut state, json!({
        "action_type": "shop_list_rarity",
        "shop_item_kind": "weapon", "shop_rarity": "rare", "price": 4, "quantity": 1
    })).unwrap();
    assert_eq!(state.shop.len(), 2);
}

#[test]
fn shop_list_item_rejects_weapon_and_armor_but_allows_consumable() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    for name in ["青钢剑", "旧皮甲"] {
        let results = director_dispatch(&mut state, json!({
            "action_type": "shop_list_item", "item_name": name, "price": 1, "quantity": 1
        })).expect("dispatch ok");
        assert_eq!(results.results[0].message_type, MessageType::Info, "{} 应被拒绝", name);
        assert!(results.results[0].log_message.contains("按稀有度类目上架"));
    }
    assert!(state.shop.is_empty());
    let results = director_dispatch(&mut state, json!({
        "action_type": "shop_list_item", "item_name": "[HP10]测试药水", "price": 1, "quantity": 5
    })).unwrap();
    assert_eq!(results.results[0].message_type, MessageType::SystemNotice);
    assert_eq!(state.shop.len(), 1);
}

#[test]
fn shop_listing_deserializes_without_new_fields() {
    let l: ShopListing = serde_json::from_str(
        r#"{"id":"l1","item_name":"青钢剑","price":2,"quantity":1}"#,
    )
    .expect("旧存档条目必须能反序列化");
    assert!(l.item_kind.is_none());
    assert!(l.rarity.is_none());
}

#[test]
fn client_json_carries_rarity_listing_fields() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    director_dispatch(&mut state, json!({
        "action_type": "shop_list_rarity",
        "shop_item_kind": "armor", "shop_rarity": "common", "price": 1, "quantity": 2
    })).unwrap();
    let json = state.to_player_client_json();
    let arr = json["shop"].as_array().expect("shop 数组存在");
    assert_eq!(arr[0]["item_kind"], "armor");
    assert_eq!(arr[0]["rarity"], "common");
    // 具体物品条目不携带新字段（skip_serializing_if）
    director_dispatch(&mut state, json!({
        "action_type": "shop_list_item", "item_name": "[HP10]测试药水", "price": 1, "quantity": 1
    })).unwrap();
    let json = state.to_player_client_json();
    let exact = json["shop"].as_array().unwrap().iter()
        .find(|e| e["item_name"] == "[HP10]测试药水").unwrap();
    assert!(exact.get("item_kind").is_none());
    assert!(exact.get("rarity").is_none());
}

use royale_arena_backend::game::game_rule_engine::Item;
use royale_arena_backend::websocket::models::ShopBuyItem;

fn buy(
    state: &mut GameState,
    player_id: &str,
    listing_id: &str,
    qty: i32,
) -> royale_arena_backend::websocket::models::ActionResults {
    state
        .handle_shop_buy_action(player_id, &[ShopBuyItem { listing_id: listing_id.to_string(), quantity: qty }])
        .expect("buy dispatch ok")
}

fn list_rarity(state: &mut GameState, kind: &str, rarity: &str, price: i32, qty: i32) -> String {
    director_dispatch(state, json!({
        "action_type": "shop_list_rarity",
        "shop_item_kind": kind, "shop_rarity": rarity, "price": price, "quantity": qty
    })).unwrap();
    state.shop.iter().find(|l| l.item_kind.as_deref() == Some(kind)).unwrap().id.clone()
}

fn build_weapon_by_name(state: &GameState, name: &str) -> Item {
    state.rule_engine.create_item_from_name(name).unwrap()
}

fn buy2(
    state: &mut GameState,
    player_id: &str,
    items: &[(String, i32)],
) -> royale_arena_backend::websocket::models::ActionResults {
    let buys: Vec<ShopBuyItem> = items
        .iter()
        .map(|(id, q)| ShopBuyItem { listing_id: id.clone(), quantity: *q })
        .collect();
    state.handle_shop_buy_action(player_id, &buys).expect("buy ok")
}

#[test]
fn buy_rarity_listing_success() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    state.players.get_mut("p1").unwrap().coins = 10.0;
    let listing_id = list_rarity(&mut state, "weapon", "common", 2, 2);

    let results = buy(&mut state, "p1", &listing_id, 1);
    let p = state.players.get("p1").unwrap();
    assert_eq!(p.inventory.len(), 1);
    let item = &p.inventory[0];
    assert!(matches!(item.item_type, royale_arena_backend::game::game_rule_engine::ItemType::Weapon(_)));
    assert_eq!(item.rarity.as_deref(), Some("common"));
    assert!(["青钢剑", "铁刃短剑"].contains(&item.name.as_str()));
    assert!((p.coins - 8.0).abs() < 1e-9, "扣款 2，实际 {}", p.coins);
    assert_eq!(state.shop[0].quantity, 1, "库存减一");
    assert!(results.results[0].log_message.contains("从商店购买"));
}

#[test]
fn buy_multiple_units_draw_distinct_names() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    state.players.get_mut("p1").unwrap().coins = 10.0;
    let listing_id = list_rarity(&mut state, "weapon", "common", 2, 2);

    let _ = buy(&mut state, "p1", &listing_id, 2);
    let p = state.players.get("p1").unwrap();
    assert_eq!(p.inventory.len(), 2);
    assert_ne!(p.inventory[0].name, p.inventory[1].name, "同批购买名字互不相同");
    assert!((p.coins - 6.0).abs() < 1e-9);
    assert!(state.shop.is_empty(), "库存归零自动移除");
}

#[test]
fn buy_fails_atomically_when_pool_exhausted() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    state.players.get_mut("p1").unwrap().coins = 10.0;
    // 占满 common 武器池的两个名字
    for name in ["青钢剑", "铁刃短剑"] {
        let item = build_weapon_by_name(&state, name);
        state.players.get_mut("p1").unwrap().inventory.push(item);
    }
    let listing_id = list_rarity(&mut state, "weapon", "common", 2, 1);

    let results = buy(&mut state, "p1", &listing_id, 1);
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(results.results[0].log_message.contains("已全部在场"));
    let p = state.players.get("p1").unwrap();
    assert_eq!(p.inventory.len(), 2, "无状态变更");
    assert!((p.coins - 10.0).abs() < 1e-9);
    assert_eq!(state.shop[0].quantity, 1, "库存不变");
}

#[test]
fn mixed_exact_and_rarity_purchase() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    state.players.get_mut("p1").unwrap().coins = 10.0;
    director_dispatch(&mut state, json!({
        "action_type": "shop_list_item", "item_name": "[HP10]测试药水", "price": 1, "quantity": 3
    })).unwrap();
    let rarity_id = list_rarity(&mut state, "armor", "common", 2, 1);
    let exact_id = state.shop.iter().find(|l| l.item_kind.is_none()).unwrap().id.clone();

    let _ = buy2(&mut state, "p1", &[(rarity_id, 1), (exact_id, 2)]);
    let p = state.players.get("p1").unwrap();
    assert_eq!(p.inventory.len(), 3, "1 件随机防具 + 2 瓶药水");
    assert_eq!(p.inventory.iter().filter(|i| matches!(i.item_type, royale_arena_backend::game::game_rule_engine::ItemType::Armor(_))).count(), 1);
    assert_eq!(p.inventory.iter().filter(|i| i.name == "[HP10]测试药水").count(), 2);
    assert!((p.coins - 6.0).abs() < 1e-9, "2 + 1×2 = 4，实际 {}", p.coins);
}

#[test]
fn legacy_exact_weapon_listing_still_buyable() {
    let mut state = build_shop_rarity_state();
    shop_add_player(&mut state, "p1", "玩家一");
    state.players.get_mut("p1").unwrap().coins = 5.0;
    state.shop.push(ShopListing {
        id: "legacy-w1".to_string(),
        item_name: "青钢剑".to_string(),
        price: 1,
        quantity: 1,
        item_kind: None,
        rarity: None,
    });
    let _ = buy(&mut state, "p1", "legacy-w1", 1);
    let p = state.players.get("p1").unwrap();
    assert_eq!(p.inventory.len(), 1);
    assert_eq!(p.inventory[0].name, "青钢剑");
    assert!((p.coins - 4.0).abs() < 1e-9);
}
