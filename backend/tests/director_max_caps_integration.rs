//! 导演设置玩家上限（max_life / max_strength / max_backpack）集成测试
//! 覆盖 clamp 收敛、当前值压限、背包降容不丢物品、拾取拦截、缺参报错

use royale_arena_backend::websocket::actions::director_action_scheduler::{
    DirectorActionParams, DirectorActionScheduler,
};
use royale_arena_backend::websocket::models::{
    GameState, Place, Player, SearchResult, SearchResultType,
};
use serde_json::{Value, json};

/// 测试规则（基础：max_life 100 / max_strength 100 / max_backpack_items 6，cap 缺省 300/300/12）
fn get_test_rules() -> Value {
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
                    {"name": "[HP上限+20]养生丸", "properties": {"effect_type": "max_life", "effect_value": 20}}
                ]
            },
            "upgrade_recipes": {}
        }
    })
}

/// 覆盖三个 cap 字段的规则变体
fn rules_with_caps(max_life_cap: i32, max_strength_cap: i32, backpack_cap: usize) -> Value {
    let mut rules = get_test_rules();
    rules["player"]["max_life_cap"] = json!(max_life_cap);
    rules["player"]["max_strength_cap"] = json!(max_strength_cap);
    rules["player"]["max_backpack_items_cap"] = json!(backpack_cap);
    rules
}

fn add_test_player(
    game_state: &mut GameState,
    player_id: &str,
    player_name: &str,
    location: &str,
) {
    game_state
        .places
        .entry(location.to_string())
        .or_insert_with(|| Place::new(location.to_string()));

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

/// 测试：max_life 超过硬上限收敛到 cap，低于基础值收敛到 base
#[test]
fn test_max_life_clamps_between_base_and_cap() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(150, 300, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    state
        .handle_set_player_max_life("p1", 250)
        .expect("set max life");
    assert_eq!(state.players["p1"].max_life, 150, "超上限应收敛到 cap 150");

    state
        .handle_set_player_max_life("p1", 50)
        .expect("set max life");
    assert_eq!(state.players["p1"].max_life, 100, "低于基础值应收敛到 base 100");
}

/// 测试：cap 配置低于 base 时，生效上限为 max(cap, base)
#[test]
fn test_max_life_effective_cap_at_least_base() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(50, 300, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    state
        .handle_set_player_max_life("p1", 80)
        .expect("set max life");
    assert_eq!(
        state.players["p1"].max_life, 100,
        "生效上限应为 max(50, 100) = 100"
    );
}

/// 测试：降低 max_life 后当前生命压到新上限
#[test]
fn test_lower_max_life_presses_current_life() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(300, 300, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    state
        .handle_set_player_max_life("p1", 200)
        .expect("raise max life");
    state.players.get_mut("p1").unwrap().life = 180;

    state
        .handle_set_player_max_life("p1", 150)
        .expect("lower max life");
    assert_eq!(state.players["p1"].max_life, 150);
    assert_eq!(state.players["p1"].life, 150, "当前生命应压到新上限");
}

/// 测试：降低 max_strength 后当前体力压到新上限（与 max_life 同构）
#[test]
fn test_lower_max_strength_presses_current_strength() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(300, 200, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    state
        .handle_set_player_max_strength("p1", 180)
        .expect("raise max strength");
    state.players.get_mut("p1").unwrap().strength = 160;

    state
        .handle_set_player_max_strength("p1", 120)
        .expect("lower max strength");
    assert_eq!(state.players["p1"].max_strength, 120);
    assert_eq!(state.players["p1"].strength, 120, "当前体力应压到新上限");
}

/// 测试：降低 max_backpack_items 不丢物品，后续拾取按新上限拦截
#[test]
fn test_lower_max_backpack_keeps_items_and_blocks_pickup() {
    let mut state = GameState::new("g1".to_string(), get_test_rules());
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    // 升到 10，塞入 8 件物品
    state
        .handle_set_player_max_backpack("p1", 10)
        .expect("raise backpack cap");
    for _ in 0..8 {
        let item = state
            .rule_engine
            .create_item_from_name("[HP上限+20]养生丸")
            .unwrap();
        state.players.get_mut("p1").unwrap().inventory.push(item);
    }
    assert_eq!(state.players["p1"].get_total_item_count(), 8);

    // 降回基础值 6：物品不丢弃
    state
        .handle_set_player_max_backpack("p1", 6)
        .expect("lower backpack cap");
    assert_eq!(state.players["p1"].max_backpack_items, 6);
    assert_eq!(
        state.players["p1"].get_total_item_count(),
        8,
        "降低上限不应丢弃已有物品"
    );

    // 地点放物品，拾取应被拦截
    let item = state
        .rule_engine
        .create_item_from_name("[HP上限+20]养生丸")
        .unwrap();
    state.places.get_mut("位置1").unwrap().items.push(item);
    set_search_result_to_last_place_item(&mut state, "p1", "位置1");
    state.handle_pick_action("p1").expect("pick returns info result");
    assert_eq!(
        state.players["p1"].get_total_item_count(),
        8,
        "超上限拾取应被拦截"
    );
}

/// 测试：值未变化（含被 clamp 收敛回原值）返回 info 消息
#[test]
fn test_set_same_max_returns_info() {
    let mut state = GameState::new("g1".to_string(), get_test_rules());
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    let results = state
        .handle_set_player_max_life("p1", 100)
        .expect("info result");
    assert_eq!(results.results.len(), 1);
    assert_eq!(state.players["p1"].max_life, 100);
}

/// 测试：玩家不存在报错
#[test]
fn test_set_max_life_player_not_found() {
    let mut state = GameState::new("g1".to_string(), get_test_rules());
    let err = state
        .handle_set_player_max_life("nope", 150)
        .unwrap_err();
    assert_eq!(err, "Player not found");
}

/// 测试：调度器缺参报错（三个动作 + 缺 player_id）
#[test]
fn test_scheduler_missing_params() {
    let mut state = GameState::new("g1".to_string(), get_test_rules());
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    let params = DirectorActionParams::from_json(&json!({"player_id": "p1"})).unwrap();
    let err = DirectorActionScheduler::dispatch(&mut state, "max_life", params).unwrap_err();
    assert_eq!(err, "Missing max_life parameter");

    let params = DirectorActionParams::from_json(&json!({"player_id": "p1"})).unwrap();
    let err = DirectorActionScheduler::dispatch(&mut state, "max_strength", params).unwrap_err();
    assert_eq!(err, "Missing max_strength parameter");

    let params = DirectorActionParams::from_json(&json!({"player_id": "p1"})).unwrap();
    let err = DirectorActionScheduler::dispatch(&mut state, "max_backpack", params).unwrap_err();
    assert_eq!(err, "Missing max_backpack_items parameter");

    let params = DirectorActionParams::from_json(&json!({"max_life": 150})).unwrap();
    let err = DirectorActionScheduler::dispatch(&mut state, "max_life", params).unwrap_err();
    assert_eq!(err, "Missing player_id parameter");
}

/// 测试：调度器端到端 —— max_life 动作经 dispatch 生效并复用 clamp
#[test]
fn test_scheduler_max_life_applies() {
    let mut state = GameState::new("g1".to_string(), rules_with_caps(150, 300, 12));
    add_test_player(&mut state, "p1", "玩家1", "位置1");

    let params =
        DirectorActionParams::from_json(&json!({"player_id": "p1", "max_life": 250})).unwrap();
    DirectorActionScheduler::dispatch(&mut state, "max_life", params).expect("dispatch ok");
    assert_eq!(
        state.players["p1"].max_life, 150,
        "dispatch 应复用 clamp 语义"
    );
}
