//! 队友系统功能集成测试

use royale_arena_backend::game::models::MessageType;
use royale_arena_backend::websocket::actions::director_action_scheduler::{
    DirectorActionParams, DirectorActionScheduler,
};
use royale_arena_backend::websocket::models::{
    GameState, Place, Player, SearchResult, SearchResultType,
};
use serde_json::{Value, json};

fn teammate_test_rules(mode: i32) -> Value {
    json!({
        "map": {"places": ["loc"], "safe_places": []},
        "player": {"max_life": 100, "max_strength": 100, "daily_life_recovery": 0, "daily_strength_recovery": 40, "search_cooldown": 30, "max_backpack_items": 6, "unarmed_damage": 5},
        "action_costs": {"move": 5, "search": 5, "pick": 0, "attack": 0, "equip": 0, "use": 0, "throw": 0, "deliver": 10},
        "rest_mode": {"life_recovery": 25, "strength_recovery": 1000, "max_moves": 1},
        "death_item_disposition": "killer_takes_loot",
        "teammate_behavior": mode,
        "items_config": {"rarity_levels": [], "items": {}, "upgrade_recipes": {}}
    })
}

fn build_empty_game_state(mode: i32) -> GameState {
    GameState::new("game1".to_string(), teammate_test_rules(mode))
}

fn extract_mode_from_rules(state: &GameState) -> i32 {
    state
        .rules_config
        .get("teammate_behavior")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32
}

#[test]
fn set_teammate_behavior_updates_rule_engine_and_rules_config() {
    let mut state = build_empty_game_state(0);
    let params = DirectorActionParams {
        sell_rarity: None,
        sell_price: None,
        shop_item_kind: None,
        shop_rarity: None,
        teammate_behavior: Some(11),
        timestamp: None,
        place_name: None,
        is_destroyed: None,
        places: None,
        weather: None,
        player_id: None,
        life: None,
        strength: None,
        coins: None,
        target_place: None,
        action_type: None,
        rest_enabled: None,
        target_type: None,
        item_name: None,
        message: None,
        airdrops: None,
        deletions: None,
        clear_all: None,
        shop_listing_id: None,
        price: None,
        quantity: None,
        max_life: None,
        max_strength: None,
        max_backpack_items: None,
    };
    let results = DirectorActionScheduler::dispatch(&mut state, "set_teammate_behavior", params)
        .expect("dispatch should succeed");

    assert_eq!(state.rule_engine.teammate_behavior.mode, 11);
    assert_eq!(extract_mode_from_rules(&state), 11);

    // Should produce one SystemNotice broadcasting to (empty) player list
    assert_eq!(results.results.len(), 1);
    let r = &results.results[0];
    assert_eq!(r.message_type, MessageType::SystemNotice);
    assert!(r.broadcast_to_director);
}

#[test]
fn set_teammate_behavior_zero_disables_mode() {
    let mut state = build_empty_game_state(11);
    let params = DirectorActionParams {
        sell_rarity: None,
        sell_price: None,
        shop_item_kind: None,
        shop_rarity: None,
        teammate_behavior: Some(0),
        timestamp: None,
        place_name: None,
        is_destroyed: None,
        places: None,
        weather: None,
        player_id: None,
        life: None,
        strength: None,
        coins: None,
        target_place: None,
        action_type: None,
        rest_enabled: None,
        target_type: None,
        item_name: None,
        message: None,
        airdrops: None,
        deletions: None,
        clear_all: None,
        shop_listing_id: None,
        price: None,
        quantity: None,
        max_life: None,
        max_strength: None,
        max_backpack_items: None,
    };
    DirectorActionScheduler::dispatch(&mut state, "set_teammate_behavior", params)
        .expect("dispatch should succeed");
    assert_eq!(state.rule_engine.teammate_behavior.mode, 0);
    assert!(!state.rule_engine.teammate_behavior.is_damage_immune());
}

#[test]
fn set_teammate_behavior_rejects_out_of_range() {
    let mut state = build_empty_game_state(0);
    for bad in [16_i32, -1, 100] {
        let params = DirectorActionParams {
            sell_rarity: None,
            sell_price: None,
            shop_item_kind: None,
            shop_rarity: None,
            teammate_behavior: Some(bad),
            timestamp: None,
            place_name: None,
            is_destroyed: None,
            places: None,
            weather: None,
            player_id: None,
            life: None,
            strength: None,
            coins: None,
            target_place: None,
            action_type: None,
            rest_enabled: None,
            target_type: None,
            item_name: None,
            message: None,
            airdrops: None,
            deletions: None,
            clear_all: None,
            shop_listing_id: None,
            price: None,
            quantity: None,
            max_life: None,
            max_strength: None,
            max_backpack_items: None,
        };
        let result = DirectorActionScheduler::dispatch(&mut state, "set_teammate_behavior", params);
        assert!(result.is_err(), "mode={} should be rejected", bad);
    }
}

// ============================================================================
// Task 3: Attack main-target immunity helpers
// ============================================================================

fn add_player_at(state: &mut GameState, id: &str, team: u32, loc: &str) {
    let mut p = Player::new(
        id.to_string(),
        format!("name_{}", id),
        "pw".to_string(),
        team,
        &state.rule_engine,
    );
    p.location = loc.to_string();
    p.last_search_result = Some(SearchResult {
        target_type: SearchResultType::Player,
        target_id: "target".to_string(),
        target_name: "target".to_string(),
        is_visible: true,
    });
    state
        .places
        .entry(loc.to_string())
        .or_insert_with(|| Place::new(loc.to_string()));
    state.players.insert(id.to_string(), p);
    state
        .places
        .get_mut(loc)
        .unwrap()
        .players
        .push(id.to_string());
}

fn set_last_search_to(state: &mut GameState, searcher: &str, target: &str) {
    let target_name = state
        .players
        .get(target)
        .map(|p| p.name.clone())
        .unwrap_or_default();
    let player = state.players.get_mut(searcher).unwrap();
    player.last_search_result = Some(SearchResult {
        target_type: SearchResultType::Player,
        target_id: target.to_string(),
        target_name,
        is_visible: true,
    });
}

#[test]
fn attack_teammate_with_damage_immune_returns_info() {
    let mut state = build_empty_game_state(1); // bit 1 on
    add_player_at(&mut state, "attacker", 1, "loc");
    add_player_at(&mut state, "target", 1, "loc");
    set_last_search_to(&mut state, "attacker", "target");

    let attacker_strength_before = state.players["attacker"].strength;
    let target_life_before = state.players["target"].life;

    let results = state.handle_attack_action("attacker").expect("attack ok");

    // Should return Info message
    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(!results.results[0].broadcast_to_director);

    // Nothing consumed, no damage applied
    assert_eq!(state.players["attacker"].strength, attacker_strength_before);
    assert_eq!(state.players["target"].life, target_life_before);
    // Search result should NOT be cleared (attack didn't happen)
    assert!(state.players["attacker"].last_search_result.is_some());
}

#[test]
fn attack_solo_player_works_even_with_mode_on() {
    let mut state = build_empty_game_state(1); // bit 1 on
    add_player_at(&mut state, "attacker", 1, "loc");
    add_player_at(&mut state, "target", 0, "loc"); // solo, team_id=0
    set_last_search_to(&mut state, "attacker", "target");

    let results = state.handle_attack_action("attacker").expect("attack ok");
    // Solo target should be damaged normally — not Info
    let has_system = results
        .results
        .iter()
        .any(|r| r.message_type == MessageType::SystemNotice);
    assert!(
        has_system,
        "attack on solo should produce SystemNotice, not Info"
    );
    assert!(state.players["target"].life < 100);
}

#[test]
fn attack_teammate_with_mode_off_works_normally() {
    let mut state = build_empty_game_state(0); // mode off
    add_player_at(&mut state, "attacker", 1, "loc");
    add_player_at(&mut state, "target", 1, "loc");
    set_last_search_to(&mut state, "attacker", "target");

    state.handle_attack_action("attacker").expect("attack ok");
    assert!(
        state.players["target"].life < 100,
        "damage should apply when mode is off"
    );
}

// ============================================================================
// Task 4: AOE splash + remote mine teammate immunity
// ============================================================================

use royale_arena_backend::game::game_rule_engine::{
    Item, ItemType, UtilityProperties, WeaponProperties,
};
use royale_arena_backend::websocket::actions::player_action_scheduler::ActionParams;

fn equip_legendary_weapon(state: &mut GameState, player_id: &str) {
    // 橙武带 aoe_damage=40, bleed_damage=10
    let weapon = Item {
        id: "w1".to_string(),
        name: "[橙]自然之力.晓".to_string(),
        internal_name: None,
        rarity: None,
        item_type: ItemType::Weapon(WeaponProperties {
            damage: 50,
            uses: Some(5),
            votes: 0,
            aoe_damage: Some(40),
            bleed_damage: Some(10),
        }),
    };
    state.players.get_mut(player_id).unwrap().equipped_weapon = Some(weapon);
}

#[test]
fn attack_splash_skips_teammates() {
    let mut state = build_empty_game_state(1); // bit 1 on
    add_player_at(&mut state, "attacker", 1, "loc");
    add_player_at(&mut state, "main_target", 2, "loc"); // 不同队，主目标
    add_player_at(&mut state, "teammate", 1, "loc"); // 同队，应被溅射过滤
    equip_legendary_weapon(&mut state, "attacker");
    set_last_search_to(&mut state, "attacker", "main_target");

    let main_life_before = state.players["main_target"].life;
    let team_life_before = state.players["teammate"].life;

    let results = state.handle_attack_action("attacker").expect("attack ok");

    // main_target 应受伤（武器 + 溅射）
    assert!(state.players["main_target"].life < main_life_before);
    // teammate 生命值不变
    assert_eq!(state.players["teammate"].life, team_life_before);

    // 攻击者收到的消息里 aoe_hits 不含 teammate
    let attacker_msg = results
        .results
        .iter()
        .find(|r| r.broadcast_players.iter().any(|p| p == "attacker"))
        .map(|r| r.log_message.as_str())
        .unwrap_or("");
    assert!(
        !attacker_msg.contains("name_teammate"),
        "teammate name should not appear in attacker log"
    );
}

#[test]
fn remote_mine_skips_teammates() {
    let mut state = build_empty_game_state(1);
    add_player_at(&mut state, "miner", 1, "loc");
    add_player_at(&mut state, "enemy", 2, "loc");
    add_player_at(&mut state, "friend", 1, "loc");

    // 给 miner 装备遥控地雷
    let mine = Item {
        id: "m1".to_string(),
        name: "[炸]遥控地雷".to_string(),
        internal_name: None,
        rarity: None,
        item_type: ItemType::Utility(UtilityProperties {
            category: "utility_trap".to_string(),
            damage: Some(30),
            uses: Some(1),
            votes: Some(0),
            targets: None,
            uses_night: None,
        }),
    };
    state.players.get_mut("miner").unwrap().inventory.push(mine);

    let enemy_before = state.players["enemy"].life;
    let friend_before = state.players["friend"].life;

    let params = ActionParams {
        target_place: None,
        place_name: None,
        item_id: Some("m1".to_string()),
        item_ids: None,
        slot_type: None,
        target_player_id: None,
        target_player_ids: None,
        target_item_name: None,
        message: None,
        shop_buy_items: None,
    };
    // 直接走 handle_use_action（绕过 scheduler 校验，简化测试）
    state
        .handle_use_action("miner", "m1", &params)
        .expect("use ok");

    assert!(
        state.players["enemy"].life < enemy_before,
        "enemy takes mine damage"
    );
    assert_eq!(
        state.players["friend"].life, friend_before,
        "friend unharmed"
    );
}

// ============================================================================
// Task 5: Search filter for teammates when bit 2 set
// ============================================================================

use royale_arena_backend::websocket::models::SearchTarget;

#[test]
fn search_filters_out_teammates_when_bit2_set() {
    let mut state = build_empty_game_state(2); // bit 2 on
    add_player_at(&mut state, "searcher", 1, "loc");
    add_player_at(&mut state, "enemy", 2, "loc");
    add_player_at(&mut state, "friend", 1, "loc");

    let targets = state.collect_search_targets("searcher");

    let target_ids: Vec<String> = targets
        .iter()
        .filter_map(|t| match t {
            SearchTarget::Player(id) => Some(id.clone()),
            _ => None,
        })
        .collect();

    assert!(
        target_ids.iter().any(|id| id == "enemy"),
        "enemy should be searchable"
    );
    assert!(
        !target_ids.iter().any(|id| id == "friend"),
        "teammate should be filtered"
    );
}

#[test]
fn search_keeps_teammates_when_bit2_off() {
    let mut state = build_empty_game_state(0); // mode off
    add_player_at(&mut state, "searcher", 1, "loc");
    add_player_at(&mut state, "friend", 1, "loc");

    let targets = state.collect_search_targets("searcher");
    let has_friend = targets
        .iter()
        .any(|t| matches!(t, SearchTarget::Player(id) if id == "friend"));
    assert!(has_friend, "teammate searchable when mode off");
}

#[test]
fn search_keeps_teammates_for_solo_searcher_even_when_bit2_on() {
    let mut state = build_empty_game_state(2);
    add_player_at(&mut state, "solo", 0, "loc"); // 散人, team_id=0
    add_player_at(&mut state, "anyone", 1, "loc");

    let targets = state.collect_search_targets("solo");
    // solo has no teammates, so anyone should be searchable
    let has_anyone = targets
        .iter()
        .any(|t| matches!(t, SearchTarget::Player(id) if id == "anyone"));
    assert!(has_anyone);
}

// ============================================================================
// Task 6: transfer_item player action
// ============================================================================

fn put_item_in_inventory(state: &mut GameState, player_id: &str, item_id: &str, item_name: &str) {
    let item = Item {
        id: item_id.to_string(),
        name: item_name.to_string(),
        internal_name: None,
        rarity: None,
        item_type: ItemType::Upgrader,
    };
    state
        .players
        .get_mut(player_id)
        .unwrap()
        .inventory
        .push(item);
}

fn transfer_params(item_id: &str, target_id: &str) -> ActionParams {
    ActionParams {
        target_place: None,
        place_name: None,
        item_id: Some(item_id.to_string()),
        item_ids: None,
        slot_type: None,
        target_player_id: Some(target_id.to_string()),
        target_player_ids: None,
        target_item_name: None,
        message: None,
        shop_buy_items: None,
    }
}

#[test]
fn transfer_item_happy_path() {
    let mut state = build_empty_game_state(8); // bit 8 on
    add_player_at(&mut state, "sender", 1, "loc");
    add_player_at(&mut state, "receiver", 1, "loc");
    put_item_in_inventory(&mut state, "sender", "i1", "[橙]自然之力.晓");

    let receiver_strength_before = state.players["receiver"].strength;
    let sender_count_before = state.players["sender"].inventory.len();

    let results = state
        .handle_transfer_item_action("sender", "i1", "receiver")
        .expect("transfer ok");

    assert!(
        state.players["sender"]
            .inventory
            .iter()
            .all(|i| i.id != "i1"),
        "item removed from sender"
    );
    assert!(
        state.players["receiver"]
            .inventory
            .iter()
            .any(|i| i.id == "i1"),
        "item added to receiver"
    );
    assert_eq!(
        state.players["sender"].inventory.len(),
        sender_count_before - 1
    );
    assert_eq!(
        state.players["receiver"].strength,
        receiver_strength_before - 5
    );
    // 成功时返回三条 SystemNotice：发起方、接收方（不广播导演）、导演专属（无目标玩家）
    assert_eq!(results.results.len(), 3);
    for r in &results.results {
        assert_eq!(r.message_type, MessageType::SystemNotice);
    }
    let sender_msg = &results.results[0];
    assert_eq!(sender_msg.broadcast_players, vec!["sender".to_string()]);
    assert!(!sender_msg.broadcast_to_director);
    let target_msg = &results.results[1];
    assert_eq!(target_msg.broadcast_players, vec!["receiver".to_string()]);
    assert!(!target_msg.broadcast_to_director);
    let director_msg = &results.results[2];
    for result in &results.results {
        assert_eq!(result.data["item_name"], "[橙]自然之力.晓");
    }
    assert_eq!(sender_msg.data["target"], director_msg.data["target"]);
    assert_eq!(target_msg.data["sender"], director_msg.data["sender"]);
    assert!(director_msg.broadcast_players.is_empty());
    assert!(director_msg.broadcast_to_director);
    assert!(director_msg.log_message.contains("玩家"));
    assert!(director_msg.log_message.contains("转移了物品"));
}

#[test]
fn transfer_item_rejected_when_bit8_off() {
    let mut state = build_empty_game_state(1); // bit 1 only
    add_player_at(&mut state, "sender", 1, "loc");
    add_player_at(&mut state, "receiver", 1, "loc");
    put_item_in_inventory(&mut state, "sender", "i1", "X");

    let results = state
        .handle_transfer_item_action("sender", "i1", "receiver")
        .expect("ok");
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        state.players["sender"]
            .inventory
            .iter()
            .any(|i| i.id == "i1"),
        "item stays"
    );
}

#[test]
fn transfer_item_rejected_when_target_low_strength() {
    let mut state = build_empty_game_state(8);
    add_player_at(&mut state, "sender", 1, "loc");
    add_player_at(&mut state, "receiver", 1, "loc");
    state.players.get_mut("receiver").unwrap().strength = 3;
    put_item_in_inventory(&mut state, "sender", "i1", "X");

    let results = state
        .handle_transfer_item_action("sender", "i1", "receiver")
        .expect("ok");
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        state.players["sender"]
            .inventory
            .iter()
            .any(|i| i.id == "i1")
    );
    assert_eq!(state.players["receiver"].strength, 3);
}

#[test]
fn transfer_item_rejected_when_target_dead() {
    let mut state = build_empty_game_state(8);
    add_player_at(&mut state, "sender", 1, "loc");
    add_player_at(&mut state, "receiver", 1, "loc");
    state.players.get_mut("receiver").unwrap().is_alive = false;
    put_item_in_inventory(&mut state, "sender", "i1", "X");

    let results = state
        .handle_transfer_item_action("sender", "i1", "receiver")
        .expect("ok");
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        state.players["sender"]
            .inventory
            .iter()
            .any(|i| i.id == "i1")
    );
}

#[test]
fn transfer_item_rejected_when_target_backpack_full() {
    let mut rules = teammate_test_rules(8);
    rules["player"]["max_backpack_items"] = json!(1);
    let mut state = GameState::new("g1".to_string(), rules);
    // manually place players
    let mut s = Player::new(
        "sender".to_string(),
        "ns".to_string(),
        "p".to_string(),
        1,
        &state.rule_engine,
    );
    s.location = "loc".to_string();
    state.players.insert("sender".to_string(), s);
    let mut r = Player::new(
        "receiver".to_string(),
        "nr".to_string(),
        "p".to_string(),
        1,
        &state.rule_engine,
    );
    r.location = "loc".to_string();
    r.inventory.push(Item {
        id: "blocker".to_string(),
        name: "B".to_string(),
        internal_name: None,
        rarity: None,
        item_type: ItemType::Upgrader,
    });
    state.players.insert("receiver".to_string(), r);
    state
        .places
        .insert("loc".to_string(), Place::new("loc".to_string()));
    put_item_in_inventory(&mut state, "sender", "i1", "X");

    let results = state
        .handle_transfer_item_action("sender", "i1", "receiver")
        .expect("ok");
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        state.players["sender"]
            .inventory
            .iter()
            .any(|i| i.id == "i1")
    );
}

#[test]
fn transfer_item_rejected_for_solo_player() {
    let mut state = build_empty_game_state(8);
    add_player_at(&mut state, "solo", 0, "loc"); // 散人
    add_player_at(&mut state, "anyone", 1, "loc");
    put_item_in_inventory(&mut state, "solo", "i1", "X");

    let results = state
        .handle_transfer_item_action("solo", "i1", "anyone")
        .expect("ok");
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(state.players["solo"].inventory.iter().any(|i| i.id == "i1"));
}

#[test]
fn transfer_item_rejected_when_item_not_in_inventory() {
    let mut state = build_empty_game_state(8);
    add_player_at(&mut state, "sender", 1, "loc");
    add_player_at(&mut state, "receiver", 1, "loc");
    // No item put in inventory

    let results = state
        .handle_transfer_item_action("sender", "ghost_item", "receiver")
        .expect("ok");
    assert_eq!(results.results[0].message_type, MessageType::Info);
}

#[test]
fn transfer_item_rejected_when_not_teammates() {
    let mut state = build_empty_game_state(8);
    add_player_at(&mut state, "sender", 1, "loc");
    add_player_at(&mut state, "other", 2, "loc"); // 不同队伍
    put_item_in_inventory(&mut state, "sender", "i1", "X");

    let results = state
        .handle_transfer_item_action("sender", "i1", "other")
        .expect("ok");
    assert_eq!(results.results[0].message_type, MessageType::Info);
    assert!(
        state.players["sender"]
            .inventory
            .iter()
            .any(|i| i.id == "i1")
    );
}

#[test]
fn transfer_item_via_scheduler_dispatch() {
    let mut state = build_empty_game_state(8);
    add_player_at(&mut state, "sender", 1, "loc");
    add_player_at(&mut state, "receiver", 1, "loc");
    put_item_in_inventory(&mut state, "sender", "i1", "X");

    let params = transfer_params("i1", "receiver");
    let results = royale_arena_backend::websocket::actions::player_action_scheduler::PlayerActionScheduler::dispatch(
        &mut state,
        "sender",
        "transfer_item",
        params,
    )
    .expect("dispatch ok");

    assert!(
        state.players["receiver"]
            .inventory
            .iter()
            .any(|i| i.id == "i1"),
        "item transferred via scheduler"
    );
    // 成功路径三条 SystemNotice（发起方、接收方、导演专属）
    assert_eq!(results.results.len(), 3);
    for r in &results.results {
        assert_eq!(r.message_type, MessageType::SystemNotice);
    }
}

// ============================================================================
// Task 7: Expose team_id in player list JSON (actor view)
// ============================================================================

#[test]
fn player_list_json_includes_team_id() {
    let mut state = build_empty_game_state(0);
    add_player_at(&mut state, "p1", 5, "loc");
    let player = state.players.get("p1").unwrap();
    let json = player.to_player_client_json_for_other_players();
    assert!(
        json.get("team_id").is_some(),
        "team_id must be present in player list JSON"
    );
    assert_eq!(json["team_id"].as_i64(), Some(5));
}
