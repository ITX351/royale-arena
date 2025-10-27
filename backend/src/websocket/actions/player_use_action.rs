//! 玩家使用道具行动处理（重构版）

use crate::game::game_rule_engine::{ConsumableProperties, Item, ItemType, UtilityProperties};
use crate::websocket::actions::player_action_scheduler::ActionParams;
use crate::websocket::actions::utils::{UseMode, UseOutcome, decrement_uses, format_delta};
use crate::websocket::models::{ActionResult, ActionResults, GameState};
use rand::seq::SliceRandom;
use serde_json::json;

/// 统一的道具使用结果描述
struct ItemUseOutcome {
    results: Vec<ActionResult>,
    use_outcome: Option<UseOutcome>,
    consume_strength: bool,
    reinsert_override: Option<bool>,
}

impl ItemUseOutcome {
    fn new(results: Vec<ActionResult>) -> Self {
        Self {
            results,
            use_outcome: None,
            consume_strength: true,
            reinsert_override: None,
        }
    }

    fn with_use_outcome(mut self, use_outcome: UseOutcome) -> Self {
        self.use_outcome = Some(use_outcome);
        self
    }

    fn with_reinsert(mut self, value: bool) -> Self {
        self.reinsert_override = Some(value);
        self
    }

    fn should_reinsert(&self) -> bool {
        if let Some(value) = self.reinsert_override {
            return value;
        }
        if let Some(outcome) = self.use_outcome {
            if let Some(remaining) = outcome.remaining_total {
                return remaining > 0;
            }
        }
        false
    }
}

impl GameState {
    /// 处理使用道具行动
    pub fn handle_use_action(
        &mut self,
        player_id: &str,
        item_id: &str,
        action_params: &ActionParams,
    ) -> Result<ActionResults, String> {
        let use_cost = self.rule_engine.action_costs.use_item;

        let (item_index, player_name, player_location) = {
            let player = self
                .players
                .get(player_id)
                .ok_or_else(|| "Player not found".to_string())?;

            let index = match player.inventory.iter().position(|item| item.id == item_id) {
                Some(index) => index,
                None => {
                    let action_result = ActionResult::new_info_message(
                        json!({}),
                        vec![player_id.to_string()],
                        "背包中没有该道具".to_string(),
                        false,
                    );
                    return Ok(action_result.as_results());
                }
            };

            (index, player.name.clone(), player.location.clone())
        };

        let mut item = {
            let player = self.players.get_mut(player_id).unwrap();
            player.inventory.remove(item_index)
        };

        let strength_before = self.players.get(player_id).unwrap().strength;

        let result = match &mut item.item_type {
            ItemType::Consumable(effect) => self.handle_consumable_use(
                player_id,
                &player_name,
                &item.name,
                effect,
                strength_before,
                use_cost,
            ),
            ItemType::Utility(properties) => self.handle_utility_use(
                player_id,
                &player_name,
                &player_location,
                &item.name,
                properties,
                action_params,
                strength_before,
                use_cost,
            ),
            _ => Err(format!("{} 无法通过使用行动触发", item.name)),
        };

        match result {
            Ok(outcome) => {
                let should_reinsert = outcome.should_reinsert();
                let should_consume_strength = outcome.consume_strength;
                let results = outcome.results;

                if should_consume_strength {
                    self.consume_strength(player_id, use_cost)?;
                }

                if should_reinsert {
                    self.reinsert_inventory_item(player_id, item_index, item);
                }

                Ok(ActionResults { results })
            }
            Err(message) => {
                self.reinsert_inventory_item(player_id, item_index, item);
                let action_result = ActionResult::new_info_message(
                    json!({}),
                    vec![player_id.to_string()],
                    message,
                    false,
                );
                Ok(action_result.as_results())
            }
        }
    }

    fn handle_consumable_use(
        &mut self,
        player_id: &str,
        player_name: &str,
        item_display_name: &str,
        effect: &mut ConsumableProperties,
        strength_before: i32,
        use_cost: i32,
    ) -> Result<ItemUseOutcome, String> {
        match effect.effect_type.as_str() {
            "heal" => {
                let (life_before, life_after, bleed_damage, curing_bleed) = {
                    let player = self.players.get_mut(player_id).unwrap();
                    let life_before = player.life;
                    let had_bleed = player.has_bleed_effect();
                    let cure_level = effect.cure_bleed.unwrap_or(0);
                    let mut curing_bleed = false;

                    if had_bleed && cure_level > 0 {
                        player.clear_bleed_effect();
                        curing_bleed = true;
                    }

                    let heal_amount = effect.effect_value;
                    let allow_heal = heal_amount > 0 && (!had_bleed || cure_level >= 2);

                    if allow_heal {
                        player.life += heal_amount;
                        if player.life > player.max_life {
                            player.life = player.max_life;
                        }
                    }

                    (life_before, player.life, player.bleed_damage, curing_bleed)
                };

                let life_delta = life_after - life_before;
                let strength_after = self.predict_strength_after_use(player_id, use_cost);
                let strength_delta = strength_after - strength_before;

                let mut log_message = format!(
                    "{} 使用了 {}，生命值{} ({})，体力{} ({})",
                    player_name,
                    item_display_name,
                    life_after,
                    format_delta(life_delta),
                    strength_after,
                    format_delta(strength_delta)
                );
                if curing_bleed {
                    log_message.push_str("，解除了流血");
                }

                let data = json!({
                    "life": life_after,
                    "life_delta": life_delta,
                    "bleed_damage": bleed_damage,
                    "strength": strength_after,
                    "strength_delta": strength_delta,
                });

                let result = ActionResult::new_system_message(
                    data,
                    vec![player_id.to_string()],
                    log_message,
                    true,
                );

                Ok(ItemUseOutcome::new(vec![result]).with_reinsert(false))
            }
            "strength" => {
                let (strength_after_recovery, restored_amount) = {
                    let player = self.players.get_mut(player_id).unwrap();
                    let before = player.strength;
                    player.strength += effect.effect_value;
                    if player.strength > player.max_strength {
                        player.strength = player.max_strength;
                    }
                    let after = player.strength;
                    let restored_amount = (after - before).max(0);
                    (after, restored_amount)
                };

                let strength_after = self.predict_strength_after_use(player_id, use_cost);
                let strength_delta = strength_after - strength_before;

                let mut log_message = format!(
                    "{} 使用了 {}，体力{} ({})",
                    player_name,
                    item_display_name,
                    strength_after,
                    format_delta(strength_delta)
                );
                if restored_amount > 0 {
                    log_message.push_str(&format!("，本次恢复{}", format_delta(restored_amount)));
                }

                let data = json!({
                    "strength": strength_after,
                    "strength_delta": strength_delta,
                    "restored_amount": restored_amount,
                    "strength_after_recovery": strength_after_recovery,
                });

                let result = ActionResult::new_system_message(
                    data,
                    vec![player_id.to_string()],
                    log_message,
                    true,
                );

                Ok(ItemUseOutcome::new(vec![result]).with_reinsert(false))
            }
            _ => Err(format!("消耗品 {} 没有定义效果", item_display_name)),
        }
    }

    fn handle_utility_use(
        &mut self,
        player_id: &str,
        player_name: &str,
        player_location: &str,
        item_display_name: &str,
        properties: &mut UtilityProperties,
        action_params: &ActionParams,
        strength_before: i32,
        use_cost: i32,
    ) -> Result<ItemUseOutcome, String> {
        match properties.category.as_str() {
            "utility_control" => self.handle_utility_control_electric_baton(
                player_id,
                player_name,
                player_location,
                item_display_name,
                properties,
                strength_before,
                use_cost,
            ),
            "utility_locator" => self.handle_utility_locator_heartbeat_detector(
                player_id,
                player_name,
                item_display_name,
                properties,
                action_params,
                strength_before,
                use_cost,
            ),
            "utility_revealer" => self.handle_utility_revealer_radar(
                player_id,
                player_name,
                item_display_name,
                properties,
                action_params,
                strength_before,
                use_cost,
            ),
            "utility_trap" => self.handle_utility_trap_remote_mine(
                player_id,
                player_name,
                player_location,
                item_display_name,
                properties,
                strength_before,
                use_cost,
            ),
            _ => Err(format!("当前未支持 {} 的使用效果", item_display_name)),
        }
    }

    fn handle_utility_control_electric_baton(
        &mut self,
        player_id: &str,
        player_name: &str,
        player_location: &str,
        item_display_name: &str,
        properties: &mut UtilityProperties,
        strength_before: i32,
        use_cost: i32,
    ) -> Result<ItemUseOutcome, String> {
        let use_outcome = decrement_uses(properties, UseMode::Night)?;

        let occupant_ids = self
            .places
            .get(player_location)
            .map(|place| place.players.clone())
            .unwrap_or_default();

        let mut results: Vec<ActionResult> = Vec::new();
        let mut bound_summaries: Vec<serde_json::Value> = Vec::new();
        let mut bound_names: Vec<String> = Vec::new();

        for target_id in occupant_ids {
            if target_id == player_id {
                continue;
            }

            let target_name = match self.players.get_mut(&target_id) {
                Some(target) => {
                    if !target.is_alive {
                        continue;
                    }
                    target.is_bound = true;
                    target.name.clone()
                }
                None => continue,
            };

            bound_names.push(target_name.clone());
            bound_summaries.push(json!({
                "player_id": target_id,
                "player_name": target_name,
            }));

            let victim_message = "你被电击棒击中，暂时无法行动".to_string();
            let victim_data = json!({
                "is_bound": true,
                "message": victim_message,
            });
            results.push(ActionResult::new_system_message(
                victim_data,
                vec![target_id.clone()],
                victim_message,
                false,
            ));
        }

        let strength_after = self.predict_strength_after_use(player_id, use_cost);
        let strength_delta = strength_after - strength_before;

        let mut log_message = format!("{} 使用了 {}", player_name, item_display_name);
        if bound_names.is_empty() {
            log_message.push_str("，但没有其他玩家受到影响");
        } else {
            log_message.push_str("，使 ");
            log_message.push_str(&bound_names.join("、"));
            log_message.push_str(" 被捆绑");
        }
        if let Some(remaining) = use_outcome.remaining_night {
            log_message.push_str(&format!("（本晚剩余 {} 次）", remaining));
        }

        let data = json!({
            "bound_players": bound_summaries,
            "strength": strength_after,
            "strength_delta": strength_delta,
            "uses_night_remaining": use_outcome.remaining_night,
        });

        results.push(ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            log_message,
            true,
        ));

        Ok(ItemUseOutcome::new(results)
            .with_use_outcome(use_outcome)
            .with_reinsert(true))
    }

    fn handle_utility_locator_heartbeat_detector(
        &mut self,
        player_id: &str,
        player_name: &str,
        item_display_name: &str,
        properties: &mut UtilityProperties,
        action_params: &ActionParams,
        strength_before: i32,
        use_cost: i32,
    ) -> Result<ItemUseOutcome, String> {
        let target_name = action_params
            .target_item_name
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "缺少目标道具名称".to_string())?
            .to_string();

        let use_outcome = decrement_uses(properties, UseMode::Night)?;

        let mut holder_info: Option<(String, String)> = None;
        let mut location_result: Option<String> = None;
        let mut found_in_equipment = false;
        let mut found_on_ground = false;

        for (id, player) in &self.players {
            if player.inventory.iter().any(|item| item.name == target_name) {
                holder_info = Some((id.clone(), player.name.clone()));
                location_result = Some(player.location.clone());
                break;
            }
            if let Some(weapon) = &player.equipped_weapon {
                if weapon.name == target_name {
                    holder_info = Some((id.clone(), player.name.clone()));
                    location_result = Some(player.location.clone());
                    found_in_equipment = true;
                    break;
                }
            }
            if let Some(armor) = &player.equipped_armor {
                if armor.name == target_name {
                    holder_info = Some((id.clone(), player.name.clone()));
                    location_result = Some(player.location.clone());
                    found_in_equipment = true;
                    break;
                }
            }
        }

        if location_result.is_none() {
            for (place_name, place) in &self.places {
                if place.items.iter().any(|item| item.name == target_name) {
                    location_result = Some(place_name.clone());
                    found_on_ground = true;
                    break;
                }
            }
        }

        let found = location_result.is_some();
        let strength_after = self.predict_strength_after_use(player_id, use_cost);
        let strength_delta = strength_after - strength_before;

        let location_description = location_result
            .clone()
            .map(|loc| {
                if loc.is_empty() {
                    "未知地点".to_string()
                } else {
                    loc
                }
            })
            .unwrap_or_else(|| "未找到目标".to_string());

        let mut log_message = if let Some((_, holder_name)) = &holder_info {
            format!(
                "{} 使用了 {}，定位 {} 当前由 {} 持有，位于 {}",
                player_name, item_display_name, target_name, holder_name, location_description
            )
        } else if found {
            format!(
                "{} 使用了 {}，定位 {} 在 {}",
                player_name, item_display_name, target_name, location_description
            )
        } else {
            format!(
                "{} 使用了 {}，未找到 {}",
                player_name, item_display_name, target_name
            )
        };
        if let Some(remaining) = use_outcome.remaining_night {
            log_message.push_str(&format!("（本晚剩余 {} 次）", remaining));
        }

        let data = json!({
            "target_item_name": target_name,
            "found": found,
            "location": location_result,
            "holder_player_id": holder_info.as_ref().map(|(id, _)| id.clone()),
            "holder_player_name": holder_info.as_ref().map(|(_, name)| name.clone()),
            "found_in_equipment": found_in_equipment,
            "found_on_ground": found_on_ground,
            "strength": strength_after,
            "strength_delta": strength_delta,
            "uses_night_remaining": use_outcome.remaining_night,
        });

        let result =
            ActionResult::new_system_message(data, vec![player_id.to_string()], log_message, true);

        Ok(ItemUseOutcome::new(vec![result])
            .with_use_outcome(use_outcome)
            .with_reinsert(true))
    }

    fn handle_utility_revealer_radar(
        &mut self,
        player_id: &str,
        player_name: &str,
        item_display_name: &str,
        properties: &mut UtilityProperties,
        action_params: &ActionParams,
        strength_before: i32,
        use_cost: i32,
    ) -> Result<ItemUseOutcome, String> {
        let requested_targets = action_params.target_player_ids.clone().unwrap_or_default();

        if requested_targets.is_empty() {
            return Err("缺少目标玩家".to_string());
        }

        let mut unique_targets: Vec<String> = Vec::new();
        for target in requested_targets {
            if !unique_targets.iter().any(|existing| existing == &target) {
                unique_targets.push(target);
            }
        }

        if let Some(limit) = properties.targets {
            let limit = limit.max(0) as usize;
            if limit > 0 && unique_targets.len() > limit {
                return Err(format!("最多可查询 {} 名玩家", limit));
            }
        }

        let mut summaries: Vec<serde_json::Value> = Vec::new();
        let mut log_segments: Vec<String> = Vec::new();
        let mut rng = rand::rng();
        for target_id in &unique_targets {
            let target_player = self
                .players
                .get(target_id)
                .ok_or_else(|| format!("目标玩家不存在：{}", target_id))?;

            let mut inventory_names: Vec<String> = target_player
                .inventory
                .iter()
                .map(|item| item.name.clone())
                .collect();

            if let Some(weapon) = &target_player.equipped_weapon {
                inventory_names.push(weapon.name.clone());
            }
            if let Some(armor) = &target_player.equipped_armor {
                inventory_names.push(armor.name.clone());
            }

            inventory_names.shuffle(&mut rng);

            let inventory_snapshot = inventory_names.clone();
            let inventory_display = if inventory_names.is_empty() {
                "无可见物品".to_string()
            } else {
                inventory_names.join("、")
            };

            if inventory_snapshot.is_empty() {
                log_segments.push(format!("{} 未携带可见物品", target_player.name));
            } else {
                log_segments.push(format!("{} 携带 {}", target_player.name, inventory_display));
            }

            summaries.push(json!({
                "player_id": target_id,
                "player_name": target_player.name,
                "location": target_player.location,
                "inventory_names": inventory_snapshot,
            }));
        }

        let use_outcome = decrement_uses(properties, UseMode::Night)?;

        let strength_after = self.predict_strength_after_use(player_id, use_cost);
        let strength_delta = strength_after - strength_before;

        let mut log_message = format!("{} 使用了 {}", player_name, item_display_name);
        if log_segments.is_empty() {
            log_message.push_str("，未侦查到目标玩家的携带物品");
        } else {
            log_message.push_str("，侦查结果 ");
            log_message.push_str(&log_segments.join("；"));
        }
        if let Some(remaining) = use_outcome.remaining_night {
            log_message.push_str(&format!("（本晚剩余 {} 次）", remaining));
        }

        let data = json!({
            "targets": summaries,
            "strength": strength_after,
            "strength_delta": strength_delta,
            "uses_night_remaining": use_outcome.remaining_night,
        });

        let result =
            ActionResult::new_system_message(data, vec![player_id.to_string()], log_message, true);

        Ok(ItemUseOutcome::new(vec![result])
            .with_use_outcome(use_outcome)
            .with_reinsert(true))
    }

    fn handle_utility_trap_remote_mine(
        &mut self,
        player_id: &str,
        player_name: &str,
        player_location: &str,
        item_display_name: &str,
        properties: &mut UtilityProperties,
        strength_before: i32,
        use_cost: i32,
    ) -> Result<ItemUseOutcome, String> {
        if let Some(uses) = properties.uses {
            if uses <= 0 {
                return Err("遥控地雷已失效，无法使用".to_string());
            }
        }

        let use_outcome = decrement_uses(properties, UseMode::Total)?;

        let damage = properties.damage.unwrap_or(0);
        if damage <= 0 {
            return Err("该遥控地雷未配置伤害，无法使用".to_string());
        }

        let occupant_ids = self
            .places
            .get(player_location)
            .map(|place| place.players.clone())
            .unwrap_or_default();

        let mut results: Vec<ActionResult> = Vec::new();
        let mut impact_records: Vec<(String, String, i32, i32, bool)> = Vec::new();
        let mut death_results: Vec<ActionResult> = Vec::new();

        for target_id in occupant_ids {
            if target_id == player_id {
                continue;
            }

            let mut requires_kill = false;
            let actual_damage = {
                let target = match self.players.get_mut(&target_id) {
                    Some(target) => target,
                    None => continue,
                };
                if !target.is_alive {
                    continue;
                }
                let before_life = target.life;
                target.life = target.life.saturating_sub(damage);
                let dealt = before_life - target.life;
                if dealt == 0 {
                    continue;
                }
                if target.life <= 0 && target.is_alive {
                    requires_kill = true;
                }
                dealt
            };

            if requires_kill {
                let mut kill_outcome = self.kill_player(&target_id, None, "遥控地雷爆炸")?;
                death_results.append(&mut kill_outcome.results);
            }

            let (target_name, life_after, is_alive_after) = {
                let target = self.players.get(&target_id).unwrap();
                (target.name.clone(), target.life, target.is_alive)
            };

            impact_records.push((
                target_id.clone(),
                target_name.clone(),
                actual_damage,
                life_after,
                is_alive_after,
            ));

            let mut victim_message = format!("你被遥控地雷炸伤，损失 {} 点生命值", actual_damage);
            if !is_alive_after {
                victim_message.push_str(" 并阵亡");
            }
            let victim_data = json!({
                "life": life_after,
                "is_alive": is_alive_after,
                "damage": actual_damage,
                "message": victim_message,
            });
            results.push(ActionResult::new_system_message(
                victim_data,
                vec![target_id.clone()],
                victim_message,
                false,
            ));
        }

        let strength_after = self.predict_strength_after_use(player_id, use_cost);
        let strength_delta = strength_after - strength_before;

        let mut log_message = format!("{} 引爆了 {}", player_name, item_display_name);
        if impact_records.is_empty() {
            log_message.push_str("，但没有造成伤害");
        } else {
            let mut segments: Vec<String> = Vec::new();
            for record in &impact_records {
                let mut segment = format!("{} 受到 {} 点伤害", record.1, record.2);
                if !record.4 {
                    segment.push_str("（阵亡）");
                }
                segments.push(segment);
            }
            //log_message.push_str("，影响了 ");
            log_message.push_str(&segments.join("，"));
        }
        if let Some(remaining) = use_outcome.remaining_total {
            log_message.push_str(&format!("（剩余可用次数 {}）", remaining));
        } else {
            log_message.push_str("（道具已耗尽）");
        }

        let impacts_json: Vec<serde_json::Value> = impact_records
            .iter()
            .map(|record| {
                json!({
                    "player_id": record.0,
                    "player_name": record.1,
                    "damage": record.2,
                    "life": record.3,
                    "is_alive": record.4,
                })
            })
            .collect();

        let inventory_snapshot = if use_outcome.remaining_total.unwrap_or(0) <= 0 {
            Some(self.players.get(player_id).unwrap().inventory.clone())
        } else {
            None
        };

        results.push(ActionResult::new_system_message(
            json!({
                "impacts": impacts_json,
                "strength": strength_after,
                "strength_delta": strength_delta,
                "uses_remaining": use_outcome.remaining_total,
                "inventory": inventory_snapshot,
            }),
            vec![player_id.to_string()],
            log_message,
            true,
        ));

        results.extend(death_results);

        let reinsert = use_outcome
            .remaining_total
            .map(|value| value > 0)
            .unwrap_or(false);

        Ok(ItemUseOutcome::new(results)
            .with_use_outcome(use_outcome)
            .with_reinsert(reinsert))
    }

    fn reinsert_inventory_item(&mut self, player_id: &str, item_index: usize, item: Item) {
        if let Some(player) = self.players.get_mut(player_id) {
            if item_index <= player.inventory.len() {
                player.inventory.insert(item_index, item);
            } else {
                player.inventory.push(item);
            }
        }
    }

    fn predict_strength_after_use(&self, player_id: &str, use_cost: i32) -> i32 {
        self.players
            .get(player_id)
            .map(|player| (player.strength - use_cost).max(0))
            .unwrap_or(0)
    }
}
