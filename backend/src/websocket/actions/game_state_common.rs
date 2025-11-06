//! GameState 通用逻辑实现

use std::collections::HashSet;
use std::mem;

use rand::{rng, seq::SliceRandom};

use crate::game::game_rule_engine::Item;
use crate::websocket::models::{
    ActionResult, ActionResults, GameState, SearchResult, SearchResultType, SearchTarget,
};

enum DeathDisposition {
    KillerTakes,
    DropToGround,
    Vanish,
}

impl DeathDisposition {
    fn from_rule(rule: &str) -> Self {
        match rule.trim() {
            "killer_takes_loot" => Self::KillerTakes,
            "drop_to_ground" => Self::DropToGround,
            "vanish_completely" => Self::Vanish,
            text if text.contains("缴获") => Self::KillerTakes,
            text if text.contains("消失") => Self::Vanish,
            _ => Self::DropToGround,
        }
    }
}

impl GameState {
    /// 杀死指定玩家
    pub fn kill_player(
        &mut self,
        target_player_id: &str,
        killer_id: Option<&str>,
        reason: &str,
    ) -> Result<ActionResults, String> {
        if !self.players.contains_key(target_player_id) {
            return Err("Player not found".to_string());
        }

        let was_alive = self.players[target_player_id].is_alive;
        let player_name = self.players[target_player_id].name.clone();

        if !was_alive {
            let data = serde_json::json!({
                "player_id": target_player_id,
                "player_name": player_name,
                "is_alive": false,
                "reason": reason,
                "message": "玩家已死亡"
            });
            let action_result = ActionResult::new_info_message(
                data,
                vec![target_player_id.to_string()],
                format!("{} 已经处于死亡状态", player_name),
                true,
            );
            return Ok(action_result.as_results());
        }

        let killer_name = killer_id.and_then(|id| self.players.get(id).map(|p| p.name.clone()));

        let (mut loot_items, previous_location) = {
            let player = self
                .players
                .get_mut(target_player_id)
                .ok_or_else(|| "Player not found".to_string())?;

            let mut items = Vec::new();
            if let Some(weapon) = player.equipped_weapon.take() {
                items.push(weapon);
            }
            if let Some(armor) = player.equipped_armor.take() {
                items.push(armor);
            }
            items.extend(player.inventory.drain(..));

            player.life = 0;
            player.strength = 0;
            player.is_alive = false;
            player.bleed_damage = 0;

            (items, mem::take(&mut player.location))
        };

        if let Some(player) = self.players.get_mut(target_player_id) {
            player.daily_reset(&self.rule_engine);
        }

        if !previous_location.is_empty() {
            if let Some(place) = self.places.get_mut(&previous_location) {
                place.players.retain(|id| id != target_player_id);
            }
        }

        let location_option = if previous_location.is_empty() {
            None
        } else {
            Some(previous_location.as_str())
        };

        let mut collected_item_names: Vec<String> = Vec::new();
        let mut dropped_item_names: Vec<String> = Vec::new();
        let mut vanished_item_names: Vec<String> = Vec::new();

        if !loot_items.is_empty() {
            let mut rng = rng();
            loot_items.shuffle(&mut rng);

            let raw_rule = &self.rule_engine.death_item_disposition.description;
            let mut disposition = DeathDisposition::from_rule(raw_rule);
            if killer_id.is_none() && matches!(disposition, DeathDisposition::KillerTakes) {
                disposition = DeathDisposition::DropToGround;
            }

            match disposition {
                DeathDisposition::Vanish => {
                    vanished_item_names.extend(Self::drain_item_names(&mut loot_items));
                }
                DeathDisposition::DropToGround => {
                    if let Some(location) = location_option {
                        if self.places.contains_key(location) {
                            dropped_item_names
                                .extend(self.drop_remaining_items(location, &mut loot_items));
                        } else {
                            vanished_item_names.extend(Self::drain_item_names(&mut loot_items));
                        }
                    } else {
                        vanished_item_names.extend(Self::drain_item_names(&mut loot_items));
                    }
                }
                DeathDisposition::KillerTakes => {
                    if let Some(killer_id) = killer_id {
                        if let Some(killer) = self.players.get_mut(killer_id) {
                            let max_backpack = self.rule_engine.player_config.max_backpack_items;
                            let current_total = killer.get_total_item_count();
                            let available_slots = max_backpack.saturating_sub(current_total);
                            if available_slots > 0 {
                                let take_count = available_slots.min(loot_items.len());
                                let taken_items: Vec<Item> =
                                    loot_items.drain(..take_count).collect();
                                collected_item_names
                                    .extend(taken_items.iter().map(|item| item.name.clone()));
                                killer.inventory.extend(taken_items);
                            }
                        }
                    }

                    if !loot_items.is_empty() {
                        if let Some(location) = location_option {
                            if self.places.contains_key(location) {
                                dropped_item_names
                                    .extend(self.drop_remaining_items(location, &mut loot_items));
                            } else {
                                vanished_item_names.extend(Self::drain_item_names(&mut loot_items));
                            }
                        } else {
                            vanished_item_names.extend(Self::drain_item_names(&mut loot_items));
                        }
                    }
                }
            }
        }

        let mut broadcast_players = vec![target_player_id.to_string()];
        if let Some(killer_id) = killer_id {
            if killer_id != target_player_id {
                broadcast_players.push(killer_id.to_string());
            }
        }

        let mut log_message = if let Some(killer_name) = killer_name.as_ref() {
            format!(
                "{} 被 {} 击杀（原因：{}）",
                player_name, killer_name, reason
            )
        } else {
            format!("{} 因 {} 死亡", player_name, reason)
        };

        let mut detail_segments: Vec<String> = Vec::new();
        Self::push_segment(&mut detail_segments, "缴获", &collected_item_names);
        Self::push_segment(&mut detail_segments, "掉落", &dropped_item_names);
        Self::push_segment(&mut detail_segments, "消失", &vanished_item_names);

        if !detail_segments.is_empty() {
            log_message.push_str("; ");
            log_message.push_str(&detail_segments.join("； "));
        }

        let data = serde_json::json!({
            "player_id": target_player_id,
            "player_name": player_name,
            "is_alive": false,
            "reason": reason,
            "killer_id": killer_id.map(|id| id.to_string()),
            "killer_name": killer_name,
            "location_before_death": location_option.map(|loc| loc.to_string()),
            "killer_collected_items": collected_item_names,
            "dropped_items": dropped_item_names,
            "vanished_items": vanished_item_names,
        });

        let action_result =
            ActionResult::new_system_message(data, broadcast_players, log_message, true);

        Ok(action_result.as_results())
    }

    /// 复活指定玩家
    pub fn revive_player(&mut self, player_id: &str, life: i32) -> Result<ActionResults, String> {
        if life <= 0 {
            return Err("复活生命值必须大于0".to_string());
        }

        let player_name = self
            .players
            .get(player_id)
            .map(|player| player.name.clone())
            .ok_or_else(|| "Player not found".to_string())?;

        {
            let player = self.players.get_mut(player_id).unwrap();
            player.life = life;
            player.strength = player.max_strength;
            player.is_alive = true;
            player.bleed_damage = 0;
        }

        {
            let player = self.players.get_mut(player_id).unwrap();
            player.daily_reset(&self.rule_engine);
        }

        let strength_after_reset = self.players.get(player_id).unwrap().strength;

        let data = serde_json::json!({
            "player_id": player_id,
            "player_name": player_name,
            "is_alive": true,
            "life": life,
            "strength": strength_after_reset,
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!("{} 被复活，生命值重置为 {}", player_name, life),
            true,
        );

        Ok(action_result.as_results())
    }

    pub fn consume_strength(&mut self, player_id: &str, amount: i32) -> Result<(), String> {
        let player = self.players.get_mut(player_id).unwrap();

        player.strength -= amount;

        // 边界检查：确保体力不低于0
        if player.strength < 0 {
            player.strength = 0;
        }

        Ok(())
    }

    /// 在非移动行动中终止静养状态
    pub fn end_rest_mode_for_action(&mut self, player_id: &str) {
        if let Some(player) = self.players.get_mut(player_id) {
            if player.rest_mode {
                player.rest_mode = false;
            }
        }
    }

    /// 记录一次移动并根据规则决定是否终止静养
    pub fn record_move_for_rest_mode(&mut self, player_id: &str) {
        let max_moves = self.rule_engine.rest_mode.max_moves;
        if let Some(player) = self.players.get_mut(player_id) {
            player.rest_moves_used = player.rest_moves_used.saturating_add(1);
            if player.rest_mode && player.rest_moves_used > max_moves {
                player.rest_mode = false;
            }
        }
    }

    fn drop_items_to_ground(&mut self, location: &str, items: Vec<Item>) -> Vec<String> {
        let item_names: Vec<String> = items.iter().map(|item| item.name.clone()).collect();

        if let Some(place) = self.places.get_mut(location) {
            place.items.extend(items);
        }

        item_names
    }

    fn drain_item_names(items: &mut Vec<Item>) -> Vec<String> {
        items.drain(..).map(|item| item.name).collect()
    }

    fn drop_remaining_items(&mut self, location: &str, items: &mut Vec<Item>) -> Vec<String> {
        let remaining: Vec<Item> = items.drain(..).collect();
        self.drop_items_to_ground(location, remaining)
    }

    fn push_segment(segments: &mut Vec<String>, label: &str, values: &[String]) {
        if !values.is_empty() {
            segments.push(format!("{}: {}", label, values.join(", ")));
        }
    }

    /// 汇总当前地点的所有搜索目标
    pub fn collect_search_targets(&self, player_id: &str) -> Vec<SearchTarget> {
        let mut targets = Vec::new();

        // 获取玩家当前位置
        let player_location = &self.players[player_id].location;

        if let Some(place) = self.places.get(player_location) {
            // 添加其他玩家到搜索目标
            for other_player_id in &place.players {
                if other_player_id != player_id {
                    // 只搜索存活的玩家
                    if let Some(other_player) = self.players.get(other_player_id) {
                        if other_player.is_alive {
                            targets.push(SearchTarget::Player(other_player_id.clone()));
                        }
                    }
                }
            }

            // 添加物品到搜索目标
            for item in &place.items {
                targets.push(SearchTarget::Item(item.id.clone()));
            }
        }

        targets
    }

    /// 等概率随机选择一个搜索目标
    pub fn select_random_target(&self, targets: &[SearchTarget]) -> SearchTarget {
        use rand::Rng;
        let mut rng = rand::rng();
        let index = rng.random_range(0..targets.len());
        targets[index].clone()
    }

    /// 处理空搜索结果
    pub fn handle_empty_search_result(&mut self, player_id: &str) -> Result<ActionResults, String> {
        self.clear_player_search_result(player_id);

        let (player_strength, player_last_search_time) = {
            let player = self.players.get(player_id).unwrap();
            (player.strength, player.last_search_time)
        };

        let data = serde_json::json!({
            "last_search_result": null,
            "strength": player_strength,
            "last_search_time": player_last_search_time
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "{} 搜索但没有发现任何东西",
                self.players.get(player_id).unwrap().name
            ),
            true,
        );
        Ok(action_result.as_results())
    }

    /// 处理玩家搜索结果
    pub fn handle_player_search_result(
        &mut self,
        player_id: &str,
        target_player_id: &str,
        reveal_target_name: bool,
    ) -> Result<ActionResults, String> {
        let target_player_name = {
            if let Some(target_player) = self.players.get(target_player_id) {
                target_player.name.clone()
            } else {
                return Err("Target player not found".to_string());
            }
        };

        // 更新玩家的上次搜索结果
        {
            let player = self.players.get_mut(player_id).unwrap();
            player.last_search_result = Some(SearchResult {
                target_type: SearchResultType::Player,
                target_id: target_player_id.to_string(),
                target_name: target_player_name.clone(),
                is_visible: reveal_target_name,
            });
        }

        let (player_strength, player_last_search_time) = {
            let player = self.players.get(player_id).unwrap();
            (player.strength, player.last_search_time)
        };

        let display_target_id = if reveal_target_name {
            target_player_id.to_string()
        } else {
            "unknown".to_string()
        };

        let display_target_name = if reveal_target_name {
            target_player_name.clone()
        } else {
            "未知玩家".to_string()
        };

        let data = serde_json::json!({
            "last_search_result": {
                "target_type": "player",
                "target_id": display_target_id,
                "target_name": display_target_name
            },
            "strength": player_strength,
            "last_search_time": player_last_search_time
        });

        let log_message = if reveal_target_name {
            format!(
                "{} 搜索发现了 {}",
                self.players.get(player_id).unwrap().name,
                target_player_name
            )
        } else {
            format!(
                "{} 搜索发现了未知玩家",
                self.players.get(player_id).unwrap().name
            )
        };

        let action_result =
            ActionResult::new_system_message(data, vec![player_id.to_string()], log_message, true);

        Ok(action_result.as_results())
    }

    /// 处理物品搜索结果
    pub fn handle_item_search_result(
        &mut self,
        player_id: &str,
        item_id: &str,
    ) -> Result<ActionResults, String> {
        let item_name = {
            let player = self.players.get(player_id).unwrap();

            // 查找物品名称
            if let Some(place) = self.places.get(&player.location) {
                if let Some(item) = place.items.iter().find(|item| item.id == item_id) {
                    item.name.clone()
                } else {
                    return Err("Item not found in place".to_string());
                }
            } else {
                return Err("Place not found".to_string());
            }
        };

        // 更新玩家的上次搜索结果
        {
            let player = self.players.get_mut(player_id).unwrap();
            player.last_search_result = Some(SearchResult {
                target_type: SearchResultType::Item,
                target_id: item_id.to_string(),
                target_name: item_name.clone(),
                is_visible: true,
            });
        }

        let (player_strength, player_last_search_time) = {
            let player = self.players.get(player_id).unwrap();
            (player.strength, player.last_search_time)
        };

        let data = serde_json::json!({
            "last_search_result": {
                "target_type": "item",
                "target_id": item_id,
                "target_name": item_name
            },
            "strength": player_strength,
            "last_search_time": player_last_search_time
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "{} 搜索并发现了物品 {}",
                self.players.get(player_id).unwrap().name,
                item_name
            ),
            true,
        );

        Ok(action_result.as_results())
    }

    pub fn clear_player_search_result(&mut self, player_id: &str) {
        if let Some(player) = self.players.get_mut(player_id) {
            player.last_search_result = None;
        }
    }

    pub fn collect_existing_weapons_and_armor_names(&self) -> HashSet<String> {
        let mut names = HashSet::new();

        // 从玩家身上收集武器和防具名称
        for player in self.players.values() {
            for item in &player.inventory {
                if item.is_weapon_or_armor() {
                    names.insert(item.name.clone());
                }
            }
            if let Some(weapon) = &player.equipped_weapon {
                names.insert(weapon.name.clone());
            }
            if let Some(armor) = &player.equipped_armor {
                names.insert(armor.name.clone());
            }
        }

        // 从场景地点收集武器和防具名称
        for place in self.places.values() {
            for item in &place.items {
                if item.is_weapon_or_armor() {
                    names.insert(item.name.clone());
                }
            }
        }

        names
    }

    /// 检查指定的物品名称是否已经存在于场上（玩家身上或地点中）
    pub fn check_item_name_exists(&self, item_name: &str) -> bool {
        let existing_names = self.collect_existing_weapons_and_armor_names();
        existing_names.contains(item_name)
    }
}
