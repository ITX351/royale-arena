//! 玩家使用道具行动处理

use crate::websocket::actions::utils::format_delta;
use crate::websocket::models::{ActionResult, ActionResults, GameState};

impl GameState {
    /// 处理使用道具行动
    pub fn handle_use_action(
        &mut self,
        player_id: &str,
        item_id: &str,
    ) -> Result<ActionResults, String> {
        // 使用规则引擎获取使用消耗
        let use_cost = self.rule_engine.action_costs.use_item;

        let (item_index, item_snapshot, player_name) = {
            let player = self.players.get(player_id).unwrap();
            match player.inventory.iter().position(|item| item.id == item_id) {
                Some(index) => (index, player.inventory[index].clone(), player.name.clone()),
                None => {
                    let data = serde_json::json!({});
                    let action_result = ActionResult::new_info_message(
                        data,
                        vec![player_id.to_string()],
                        "背包中没有该道具".to_string(),
                        false,
                    );
                    return Ok(action_result.as_results());
                }
            }
        };

        let item_name = item_snapshot.name.clone();
        let effect = if let Some(effect) = item_snapshot.as_consumable() {
            effect.clone()
        } else {
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "该物品不是消耗品，无法使用".to_string(),
                false,
            );
            return Ok(action_result.as_results());
        };

        {
            let player = self.players.get_mut(player_id).unwrap();
            player.inventory.remove(item_index);
        }

        // 执行效果
        match effect.effect_type.as_str() {
            "heal" => {
                // 治疗效果
                let (life_before, life_after, bleed_damage, curing_bleed) = {
                    let player = self.players.get_mut(player_id).unwrap();
                    let life_before = player.life;
                    let had_bleed = player.has_bleed_effect();
                    let cure_level = effect.cure_bleed.unwrap_or(0);
                    let mut curing_bleed = false;

                    if had_bleed && cure_level > 0 {
                        // 解除流血效果
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

                let strength_before_use = self.players.get(player_id).unwrap().strength;
                // 消耗体力值
                self.consume_strength(player_id, use_cost)?;

                let strength_after_use = self.players.get(player_id).unwrap().strength;
                let strength_delta = strength_after_use - strength_before_use;

                let mut log_message = format!(
                    "{} 使用了 {}，生命值{} ({})，体力{} ({})",
                    player_name,
                    item_name,
                    life_after,
                    format_delta(life_delta),
                    strength_after_use,
                    format_delta(strength_delta)
                );
                if curing_bleed {
                    log_message.push_str("，解除了流血");
                }

                let data = serde_json::json!({
                    "life": life_after,
                    "life_delta": life_delta,
                    "bleed_damage": bleed_damage,
                    "strength": strength_after_use,
                    "strength_delta": strength_delta
                });

                let action_result = ActionResult::new_system_message(
                    data,
                    vec![player_id.to_string()],
                    log_message,
                    true,
                );
                Ok(action_result.as_results())
            }
            "strength" => {
                // 体力恢复效果
                let (strength_before_use, strength_after_recovery, restored_amount) = {
                    let player = self.players.get_mut(player_id).unwrap();
                    let before = player.strength;
                    player.strength += effect.effect_value;
                    if player.strength > player.max_strength {
                        player.strength = player.max_strength;
                    }
                    let after = player.strength;
                    let restored_amount = (after - before).max(0);
                    (before, after, restored_amount)
                };

                // 消耗使用体力（在恢复后扣除）
                self.consume_strength(player_id, use_cost)?;

                let strength_after_use = self.players.get(player_id).unwrap().strength;
                let strength_delta = strength_after_use - strength_before_use;

                let mut log_message = format!(
                    "{} 使用了 {}，体力{} ({})",
                    player_name,
                    item_name,
                    strength_after_use,
                    format_delta(strength_delta)
                );
                if restored_amount > 0 {
                    log_message.push_str(&format!("，本次恢复{}", format_delta(restored_amount)));
                }

                let data = serde_json::json!({
                    "strength": strength_after_use,
                    "strength_delta": strength_delta,
                    "restored_amount": restored_amount,
                    "strength_after_recovery": strength_after_recovery
                });

                let action_result = ActionResult::new_system_message(
                    data,
                    vec![player_id.to_string()],
                    log_message,
                    true,
                );
                Ok(action_result.as_results())
            }
            _ => {
                // 未知效果类型
                let data = serde_json::json!({});
                let action_result = ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    format!("消耗品 {} 没有定义效果", item_name),
                    false,
                );
                Ok(action_result.as_results())
            }
        }
    }
}
