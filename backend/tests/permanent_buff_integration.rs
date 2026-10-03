//! 永久增益道具系统集成测试
//! 测试上限提升、clamp、溢出浪费、背包扩容、旧状态兼容

use royale_arena_backend::game::game_rule_engine::GameRuleEngine;
use royale_arena_backend::websocket::actions::player_action_scheduler::ActionParams;
use royale_arena_backend::websocket::models::{
    GameState, Place, Player, SearchResult, SearchResultType,
};
use serde_json::{Value, json};

/// 测试规则配置（含永久增益道具）
fn get_test_rules_with_permanent_buffs() -> Value {
    json!({
        "map": {"places": ["位置1", "位置2"], "safe_places": []},
        "player": {
            "max_life": 100,
            "max_strength": 100,
            "daily_life_recovery": 0,
            "daily_strength_recovery": 40,
            "search_cooldown": 4,
            "max_backpack_items": 6,
            "unarmed_damage": 10
        },
        "action_costs": {"move": 5, "search": 5, "pick": 0, "attack": 0, "equip": 0, "use": 0, "throw": 0, "deliver": 10},
        "rest_mode": {"life_recovery": 25, "strength_recovery": 1000, "max_moves": 1},
        "death_item_disposition": "killer_takes_loot",
        "teammate_behavior": 0,
        "items_config": {
            "rarity_levels": [],
            "items": {
                "weapons": [],
                "armors": [],
                "utilities": [],
                "consumables": [],
                "upgraders": [],
                "currencies": [],
                "permanent_buffs": [
                    {"name": "[HP上限+20]养生丸", "properties": {"effect_type": "max_life", "effect_value": 20}},
                    {"name": "[HP上限+50]壮骨丹", "properties": {"effect_type": "max_life", "effect_value": 50}},
                    {"name": "[HP上限+100]金钟罩", "properties": {"effect_type": "max_life", "effect_value": 100}},
                    {"name": "[MP上限+50]行军丹", "properties": {"effect_type": "max_strength", "effect_value": 50}},
                    {"name": "[背包+2]腰包", "properties": {"effect_type": "max_backpack", "effect_value": 2}},
                    {"name": "[背包+6]百宝袋", "properties": {"effect_type": "max_backpack", "effect_value": 6}},
                    {"name": "[HP上限-50]衰弱咒", "properties": {"effect_type": "max_life", "effect_value": -50}},
                    {"name": "[背包-2]破损袋", "properties": {"effect_type": "max_backpack", "effect_value": -2}},
                    {"name": "[异常]无效丹", "properties": {"effect_type": "unknown_x", "effect_value": 10}}
                ]
            },
            "upgrade_recipes": {}
        }
    })
}

/// 覆盖三个 cap 字段的规则变体
fn rules_with_caps(max_life_cap: i32, max_strength_cap: i32, backpack_cap: usize) -> Value {
    let mut rules = get_test_rules_with_permanent_buffs();
    rules["player"]["max_life_cap"] = json!(max_life_cap);
    rules["player"]["max_strength_cap"] = json!(max_strength_cap);
    rules["player"]["max_backpack_items_cap"] = json!(backpack_cap);
    rules
}

// 以下辅助函数供 Task 3/4 的后续测试复用，当前任务暂未使用
fn add_test_place(game_state: &mut GameState, place_name: &str) {
    game_state
        .places
        .entry(place_name.to_string())
        .or_insert_with(|| Place::new(place_name.to_string()));
}

fn add_test_player(game_state: &mut GameState, player_id: &str, player_name: &str, location: &str) {
    add_test_place(game_state, location);

    let mut player = Player::new(
        player_id.to_string(),
        player_name.to_string(),
        "password".to_string(),
        1,
        &game_state.rule_engine,
    );
    player.location = location.to_string();

    game_state.players.insert(player_id.to_string(), player);
    game_state
        .places
        .get_mut(location)
        .unwrap()
        .players
        .push(player_id.to_string());
}

/// 按名称给玩家背包塞入道具，返回道具 ID
fn give_item(game_state: &mut GameState, player_id: &str, item_name: &str) -> String {
    let item = game_state
        .rule_engine
        .create_item_from_name(item_name)
        .expect("item should exist in rules");
    let item_id = item.id.clone();
    game_state
        .players
        .get_mut(player_id)
        .unwrap()
        .inventory
        .push(item);
    item_id
}

fn empty_action_params() -> ActionParams {
    ActionParams {
        target_place: None,
        place_name: None,
        item_id: None,
        item_ids: None,
        slot_type: None,
        target_player_id: None,
        target_player_ids: None,
        target_item_name: None,
        message: None,
        shop_buy_items: None,
    }
}

#[test]
fn extreme_signed_buffs_clamp_without_overflow() {
    for (effect_type, field) in [("max_life", "max_life"), ("max_strength", "max_strength")] {
        for effect_value in [i32::MAX, i32::MIN] {
            let mut rules = rules_with_caps(300, 300, 12);
            rules["items_config"]["items"]["permanent_buffs"] = json!([{
                "name": "boundary buff",
                "properties": {"effect_type": effect_type, "effect_value": effect_value}
            }]);
            let mut state = GameState::new("boundary".to_string(), rules);
            add_test_player(&mut state, "p1", "玩家1", "位置1");
            let id = give_item(&mut state, "p1", "boundary buff");
            let results = state
                .handle_use_action("p1", &id, &empty_action_params())
                .unwrap();
            let expected = if effect_value > 0 { 300 } else { 100 };
            assert_eq!(results.results[0].data[field], json!(expected));
            assert_eq!(
                results.results[0].data[format!("{field}_delta")],
                json!(expected - 100)
            );
            assert!(state.players["p1"].inventory.is_empty());
        }
    }
}

#[test]
fn backpack_buffs_preserve_large_capacities_and_signed_deltas() {
    // Include usize::MAX to catch both narrowing casts and addition overflow.
    for (before, effect_value, expected) in [
        (i32::MAX as usize + 10, 2, i32::MAX as usize + 12),
        (usize::MAX - 1, i32::MAX, usize::MAX),
        (
            usize::MAX,
            i32::MIN,
            usize::MAX - i32::MIN.unsigned_abs() as usize,
        ),
        (6, i32::MIN, 6),
    ] {
        let mut rules = rules_with_caps(300, 300, usize::MAX);
        rules["items_config"]["items"]["permanent_buffs"] = json!([{
            "name": "backpack boundary",
            "properties": {"effect_type": "max_backpack", "effect_value": effect_value}
        }]);
        let mut state = GameState::new("boundary".to_string(), rules);
        add_test_player(&mut state, "p1", "玩家1", "位置1");
        state.players.get_mut("p1").unwrap().max_backpack_items = before;
        let id = give_item(&mut state, "p1", "backpack boundary");
        let results = state
            .handle_use_action("p1", &id, &empty_action_params())
            .unwrap();
        assert_eq!(state.players["p1"].max_backpack_items, expected);
        assert_eq!(
            results.results[0].data["max_backpack_items"],
            json!(expected)
        );
        assert_eq!(
            results.results[0].data["max_backpack_items_delta"],
            json!(expected as i128 - before as i128)
        );
        assert!(
            results.results[0]
                .log_message
                .contains(&expected.to_string())
        );
        assert!(state.players["p1"].inventory.is_empty());
    }
}

/// 把玩家上一次搜索结果指向地点中最后一个物品（供拾取测试使用）
fn set_search_result_to_last_place_item(
    game_state: &mut GameState,
    player_id: &str,
    place_name: &str,
) {
    let place = game_state.places.get(place_name).unwrap();
    let target_id = place.items.last().unwrap().id.clone();
    let target_name = place.items.last().unwrap().name.clone();
    game_state
        .players
        .get_mut(player_id)
        .unwrap()
        .last_search_result = Some(SearchResult {
        target_type: SearchResultType::Item,
        target_id,
        target_name,
        is_visible: true,
    });
}

/// 测试：cap 字段缺省时取默认值 300/300/12
#[test]
fn test_cap_defaults_when_not_configured() {
    let rules_json =
        serde_json::to_string(&get_test_rules_with_permanent_buffs()).expect("serialize rules");
    let engine = GameRuleEngine::from_json(&rules_json).expect("parse rules");
    assert_eq!(engine.player_config.max_life_cap, 300);
    assert_eq!(engine.player_config.max_strength_cap, 300);
    assert_eq!(engine.player_config.max_backpack_items_cap, 12);
}

/// 测试：cap 字段显式配置时可解析
#[test]
fn test_caps_parse_from_config() {
    let rules_json = serde_json::to_string(&rules_with_caps(250, 260, 10)).expect("serialize");
    let engine = GameRuleEngine::from_json(&rules_json).expect("parse");
    assert_eq!(engine.player_config.max_life_cap, 250);
    assert_eq!(engine.player_config.max_strength_cap, 260);
    assert_eq!(engine.player_config.max_backpack_items_cap, 10);
}

/// 测试：新玩家背包容量从规则初始化
#[test]
fn test_player_new_initializes_backpack_from_rules() {
    let rules_json =
        serde_json::to_string(&get_test_rules_with_permanent_buffs()).expect("serialize rules");
    let engine = GameRuleEngine::from_json(&rules_json).expect("parse");
    let player = Player::new(
        "p1".to_string(),
        "玩家1".to_string(),
        "pw".to_string(),
        1,
        &engine,
    );
    assert_eq!(player.max_backpack_items, 6);
    assert_eq!(player.max_life, 100);
    assert_eq!(player.max_strength, 100);
}

/// 测试：旧玩家状态（无 max_backpack_items 字段）反序列化后回填规则值
#[test]
fn test_deserialize_old_player_state_backfills_backpack() {
    let state_json = json!({
        "game_id": "g1",
        "players": {
            "p1": {
                "id": "p1",
                "name": "玩家1",
                "password": "pw",
                "location": "位置1",
                "life": 80,
                "strength": 90,
                "max_life": 100,
                "max_strength": 100,
                "inventory": [],
                "equipped_weapon": null,
                "equipped_armor": null,
                "last_search_result": null,
                "is_alive": true,
                "is_bound": false,
                "rest_mode": true,
                "rest_moves_used": 0,
                "last_search_time": null,
                "team_id": 1,
                "bleed_damage": 0,
                "coins": 0.0
            }
        },
        "places": {},
        "weather": 1.0,
        "votes": {},
        "rules_config": get_test_rules_with_permanent_buffs(),
        "night_start_time": null,
        "night_end_time": null,
        "next_night_destroyed_places": [],
        "save_time": null
    });
    let state: GameState =
        serde_json::from_value(state_json).expect("deserialize legacy game state");
    assert_eq!(state.players["p1"].max_backpack_items, 6);
}

/// 测试：使用血量上限道具——只涨上限，当前血量/体力不变，道具销毁
#[test]
fn test_use_max_life_buff_raises_cap_only() {
    let mut state = GameState::new("g1".to_string(), get_test_rules_with_permanent_buffs());
    add_test_player(&mut state, "p1", "玩家1", "位置1");
    state.players.get_mut("p1").unwrap().life = 80;

    let item_id = give_item(&mut state, "p1", "[HP上限+50]壮骨丹");
    let results = state
        .handle_use_action("p1", &item_id, &empty_action_params())
        .expect("use should succeed");

    let player = state.players.get("p1").unwrap();
    assert_eq!(player.max_life, 150);
    assert_eq!(player.life, 80, "当前血量不应变化");
    assert_eq!(player.strength, 100, "use 消耗默认为 0");
    assert!(player.inventory.is_empty(), "道具应被销毁");
    assert_eq!(results.results.len(), 1);
}

/// 测试：使用体力上限道具
#[test]
fn test_use_max_strength_buff() {
    let mut state = GameState::new("g1".to_string(), get_test_rules_with_permanent_buffs());
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    let item_id = give_item(&mut state, "p1", "[MP上限+50]行军丹");
    state
        .handle_use_action("p1", &item_id, &empty_action_params())
        .expect("use should succeed");

    let player = state.players.get("p1").unwrap();
    assert_eq!(player.max_strength, 150);
    assert_eq!(player.strength, 100, "当前体力不应变化");
    assert!(player.inventory.is_empty());
}

/// 测试：连续使用至硬上限——clamp 且溢出浪费
#[test]
fn test_use_max_life_buff_clamps_to_cap() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(150, 300, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    // 100 + 100 → clamp 150，溢出 50 浪费
    let id1 = give_item(&mut state, "p1", "[HP上限+100]金钟罩");
    state
        .handle_use_action("p1", &id1, &empty_action_params())
        .expect("use should succeed");
    assert_eq!(state.players["p1"].max_life, 150);

    // 已到 cap 再用 +50 → +0，道具照常消耗
    let id2 = give_item(&mut state, "p1", "[HP上限+50]壮骨丹");
    state
        .handle_use_action("p1", &id2, &empty_action_params())
        .expect("use should succeed");
    assert_eq!(state.players["p1"].max_life, 150);
    assert!(state.players["p1"].inventory.is_empty());
}

/// 测试：cap 低于规则基础值时永不降低上限（生效上限 = max(cap, 基础值)）
#[test]
fn test_cap_below_base_never_lowers_max() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(50, 300, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    let item_id = give_item(&mut state, "p1", "[HP上限+100]金钟罩");
    state
        .handle_use_action("p1", &item_id, &empty_action_params())
        .expect("use should succeed");
    assert_eq!(
        state.players["p1"].max_life, 100,
        "生效上限应为 max(50, 100) = 100"
    );
}

/// 测试：负值道具降低生命上限，下限为规则基础值；当前生命压到新上限
#[test]
fn test_negative_buff_lowers_cap_with_base_floor() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(300, 300, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    // 先涨：100 → 200
    let id_up = give_item(&mut state, "p1", "[HP上限+100]金钟罩");
    state
        .handle_use_action("p1", &id_up, &empty_action_params())
        .expect("use should succeed");
    assert_eq!(state.players["p1"].max_life, 200);

    // 模拟治疗后生命超过基础值
    state.players.get_mut("p1").unwrap().life = 180;

    // -50：200 → 150，生命从 180 压到 150
    let id_down = give_item(&mut state, "p1", "[HP上限-50]衰弱咒");
    state
        .handle_use_action("p1", &id_down, &empty_action_params())
        .expect("use should succeed");
    assert_eq!(state.players["p1"].max_life, 150);
    assert_eq!(state.players["p1"].life, 150, "当前生命应压到新上限");

    // 连续 -50：150 → 100（下限为规则基础值），生命同步压到 100，道具照常消耗
    for _ in 0..2 {
        let id = give_item(&mut state, "p1", "[HP上限-50]衰弱咒");
        state
            .handle_use_action("p1", &id, &empty_action_params())
            .expect("use should succeed");
    }
    assert_eq!(state.players["p1"].max_life, 100, "下限应为规则基础值 100");
    assert_eq!(state.players["p1"].life, 100);
    assert!(state.players["p1"].inventory.is_empty());
}

/// 测试：负值背包道具降低容量，下限为规则基础值
#[test]
fn test_negative_backpack_buff_floors_at_base() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(300, 300, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    // 先扩：6 → 12（cap）
    let id_up = give_item(&mut state, "p1", "[背包+6]百宝袋");
    state
        .handle_use_action("p1", &id_up, &empty_action_params())
        .expect("use should succeed");
    assert_eq!(state.players["p1"].max_backpack_items, 12);

    // 连续 -2：12 → 10 → 8 → 6 → 6（下限为规则基础值）
    for _ in 0..4 {
        let id = give_item(&mut state, "p1", "[背包-2]破损袋");
        state
            .handle_use_action("p1", &id, &empty_action_params())
            .expect("use should succeed");
    }
    assert_eq!(
        state.players["p1"].max_backpack_items, 6,
        "下限应为规则基础值 6"
    );
}

/// 测试：未知 effect_type —— 报错提示、道具回插、不扣体力
#[test]
fn test_unknown_effect_type_reinserts_item() {
    let mut state = GameState::new("g1".to_string(), get_test_rules_with_permanent_buffs());
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    let item_id = give_item(&mut state, "p1", "[异常]无效丹");
    let results = state
        .handle_use_action("p1", &item_id, &empty_action_params())
        .expect("use action returns info result");

    let player = state.players.get("p1").unwrap();
    assert_eq!(player.inventory.len(), 1, "道具应原位插回");
    assert_eq!(player.inventory[0].id, item_id);
    assert_eq!(player.strength, 100);
    assert_eq!(results.results.len(), 1);
}

/// 测试：背包扩容后可继续拾取（拾取容量检查使用玩家字段而非全局规则）
#[test]
fn test_backpack_expansion_allows_more_picks() {
    let mut state = GameState::new("g1".to_string(), get_test_rules_with_permanent_buffs());
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    // 背包先放一个扩容道具（占 1 格，基础容量 6）
    let bag_id = give_item(&mut state, "p1", "[背包+6]百宝袋");
    assert_eq!(state.players["p1"].get_total_item_count(), 1);

    // 地点放 8 个物品
    for _ in 0..8 {
        let item = state
            .rule_engine
            .create_item_from_name("[HP上限+20]养生丸")
            .unwrap();
        state.places.get_mut("位置1").unwrap().items.push(item);
    }

    // 连续拾取 5 次：容量 6 已占 1，第 6 次提示背包已满
    for _ in 0..5 {
        set_search_result_to_last_place_item(&mut state, "p1", "位置1");
        state.handle_pick_action("p1").expect("pick should succeed");
    }
    assert_eq!(state.players["p1"].get_total_item_count(), 6);

    set_search_result_to_last_place_item(&mut state, "p1", "位置1");
    state.handle_pick_action("p1").expect("pick returns info");
    assert_eq!(
        state.players["p1"].get_total_item_count(),
        6,
        "容量 6 时应拒绝拾取"
    );

    // 使用百宝袋：容量 12，道具销毁后背包 5 件
    state
        .handle_use_action("p1", &bag_id, &empty_action_params())
        .expect("use bag");
    assert_eq!(state.players["p1"].max_backpack_items, 12);
    assert_eq!(state.players["p1"].get_total_item_count(), 5);

    // 扩容后可继续拾取剩余 3 件（若容量检查仍读全局规则 6，第二件起会被拦截）
    for _ in 0..3 {
        set_search_result_to_last_place_item(&mut state, "p1", "位置1");
        state
            .handle_pick_action("p1")
            .expect("pick after expansion");
    }
    assert_eq!(state.players["p1"].get_total_item_count(), 8);
}

/// 测试：商店可按名称上架永久增益道具（现有"拒绝武器/防具"逻辑自动放行）
#[test]
fn test_shop_list_permanent_buff_by_name() {
    let mut state = GameState::new("g1".to_string(), get_test_rules_with_permanent_buffs());
    state
        .handle_shop_list_item("[HP上限+20]养生丸".to_string(), 10, 2)
        .expect("list should succeed");
    assert_eq!(state.shop.len(), 1);
    assert_eq!(state.shop[0].item_name, "[HP上限+20]养生丸");
    assert_eq!(state.shop[0].price, 10);
    assert_eq!(state.shop[0].quantity, 2);
}
