//! 玩家攻击行动处理

use crate::websocket::models::{ActionResult, ActionResults, GameState, SearchResultType};

impl GameState {
    /// 处理攻击行动
    pub fn handle_attack_action(&mut self, player_id: &str) -> Result<ActionResults, String> {
        // 使用规则引擎获取攻击消耗
        let attack_cost = self.rule_engine.action_costs.attack;

        // 检查前置条件：上一次搜索结果为玩家
        let (player_location, target_player_id) = {
            let player = self.players.get(player_id).unwrap();

            let target_player_id = if let Some(ref search_result) = player.last_search_result
                && search_result.target_type == SearchResultType::Player
            {
                search_result.target_id.clone()
            } else {
                let action_result = ActionResult::new_info_message(
                    serde_json::json!({}),
                    vec![player_id.to_string()],
                    "上一次搜索结果不是玩家".to_string(),
                    false,
                );
                return Ok(action_result.as_results());
            };

            (player.location.clone(), target_player_id)
        };

        // 阻止在安全区内发动攻击
        if self.rule_engine.is_safe_place(&player_location) {
            let action_result = ActionResult::new_info_message(
                serde_json::json!({
                    "place": player_location,
                }),
                vec![player_id.to_string()],
                "当前地点为安全区，无法发动攻击".to_string(),
                false,
            );
            return Ok(action_result.as_results());
        }

        // 获取目标玩家信息
        let (target_player_location, target_player_alive, target_player_name) = {
            let target_player = self
                .players
                .get(&target_player_id)
                .ok_or("Target player not found".to_string())?;
            (
                target_player.location.clone(),
                target_player.is_alive,
                target_player.name.clone(),
            )
        };

        // 验证目标玩家是否在同一地点
        let failed_message = "攻击目标玩家失败".to_string();
        if target_player_location != player_location {
            let action_result = ActionResult::new_info_message(
                serde_json::json!({}),
                vec![player_id.to_string()],
                failed_message,
                false,
            );
            return Ok(action_result.as_results());
        }

        // 验证目标玩家是否已死亡
        if !target_player_alive {
            let action_result = ActionResult::new_info_message(
                serde_json::json!({}),
                vec![player_id.to_string()],
                failed_message,
                false,
            );
            return Ok(action_result.as_results());
        }

        // 根据是否装备武器计算伤害及附加效果
        let (base_damage, attack_method, weapon_aoe_damage, weapon_bleed_damage) = {
            let attacker = self.players.get(player_id).unwrap();
            if let Some(weapon) = &attacker.equipped_weapon
                && let Some(attributes) = weapon.as_weapon()
            {
                let aoe = attributes.aoe_damage.filter(|value| *value > 0);
                let bleed = attributes.bleed_damage.filter(|value| *value > 0);
                (attributes.damage, "武器", aoe, bleed)
            } else {
                (self.rule_engine.get_unarmed_damage(), "挥拳", None, None)
            }
        };

        // 主要目标根据防具减免伤害
        let armor_defense = {
            let target = self.players.get(&target_player_id).unwrap();
            if let Some(armor) = &target.equipped_armor
                && let Some(attributes) = armor.as_armor()
            {
                attributes.defense
            } else {
                0
            }
        };

        let damage = (base_damage - armor_defense).max(0);

        // 预先收集可能的溅射目标
        let aoe_targets: Vec<String> = if weapon_aoe_damage.is_some() {
            if let Some(place) = self.places.get(&player_location) {
                place
                    .players
                    .iter()
                    .filter_map(|other_id| {
                        if other_id.as_str() == player_id || other_id == &target_player_id {
                            return None;
                        }
                        let is_alive = self
                            .players
                            .get(other_id)
                            .map(|player| player.is_alive)
                            .unwrap_or(false);
                        if is_alive {
                            Some(other_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let attacker_name = self.players.get(player_id).unwrap().name.clone();

        let mut weapon_destroyed_result: Option<ActionResult> = None;
        let mut armor_destroyed_result: Option<ActionResult> = None;

        // 消耗武器耐久
        {
            let mut weapon_should_remove = false;
            {
                let attacker = self.players.get_mut(player_id).unwrap();
                if let Some(weapon) = attacker.equipped_weapon.as_mut() {
                    if let crate::game::game_rule_engine::ItemType::Weapon(properties) =
                        &mut weapon.item_type
                    {
                        if let Some(uses) = properties.uses.as_mut() {
                            *uses -= 1;
                            if *uses <= 0 {
                                *uses = 0;
                                weapon_should_remove = true;
                            }
                        }
                    }
                }
            }

            if weapon_should_remove {
                let attacker = self.players.get_mut(player_id).unwrap();
                if let Some(broken_weapon) = attacker.equipped_weapon.take() {
                    let player_name = attacker.name.clone();
                    let inventory_snapshot = attacker.inventory.clone();
                    let strength_snapshot = attacker.strength;
                    let equipped_weapon_snapshot = attacker.equipped_weapon.clone();
                    let broken_weapon_name = broken_weapon.name.clone();

                    let data = serde_json::json!({
                        "equipped_weapon": equipped_weapon_snapshot,
                        "inventory": inventory_snapshot,
                        "strength": strength_snapshot
                    });

                    weapon_destroyed_result = Some(ActionResult::new_system_message(
                        data,
                        vec![player_id.to_string()],
                        format!("{} 的武器 {} 已损坏", player_name, broken_weapon_name),
                        true,
                    ));
                }
            }
        }

        // 消耗护甲耐久
        {
            let mut armor_should_remove = false;
            {
                let target = self.players.get_mut(&target_player_id).unwrap();
                if let Some(armor) = target.equipped_armor.as_mut() {
                    if let crate::game::game_rule_engine::ItemType::Armor(properties) =
                        &mut armor.item_type
                    {
                        if let Some(uses) = properties.uses.as_mut() {
                            *uses -= 1;
                            if *uses <= 0 {
                                *uses = 0;
                                armor_should_remove = true;
                            }
                        }
                    }
                }
            }

            if armor_should_remove {
                let target = self.players.get_mut(&target_player_id).unwrap();
                if let Some(broken_armor) = target.equipped_armor.take() {
                    let target_name = target.name.clone();
                    let inventory_snapshot = target.inventory.clone();
                    let strength_snapshot = target.strength;
                    let equipped_armor_snapshot = target.equipped_armor.clone();
                    let broken_armor_name = broken_armor.name.clone();

                    let data = serde_json::json!({
                        "equipped_armor": equipped_armor_snapshot,
                        "inventory": inventory_snapshot,
                        "strength": strength_snapshot
                    });

                    armor_destroyed_result = Some(ActionResult::new_system_message(
                        data,
                        vec![target_player_id.to_string()],
                        format!("{} 的防具 {} 已损坏", target_name, broken_armor_name),
                        true,
                    ));
                }
            }
        }

        // 对主目标造成伤害，并记录实际伤害与流血效果
        let mut main_bleed_value: Option<i32> = None;
        let mut main_requires_kill = false;
        let main_actual_damage = {
            let target_player = self.players.get_mut(&target_player_id).unwrap();
            let before_life = target_player.life;
            target_player.life = target_player.life.saturating_sub(damage);
            let dealt = before_life - target_player.life;

            if dealt > 0 {
                if let Some(bleed_value) = weapon_bleed_damage {
                    if target_player.update_bleed_effect(bleed_value) {
                        main_bleed_value = Some(bleed_value);
                    }
                }
                if target_player.life <= 0 && target_player.is_alive {
                    main_requires_kill = true;
                }
            }

            dealt
        };

        let mut death_results: Vec<ActionResult> = Vec::new();
        if main_requires_kill {
            let mut death_outcome =
                self.kill_player(&target_player_id, Some(player_id), "攻击致死")?;
            death_results.append(&mut death_outcome.results);
        }

        // 处理武器溅射伤害
        let mut aoe_impacts: Vec<(String, String, i32, i32, bool, Option<i32>)> = Vec::new();
        let mut aoe_results: Vec<ActionResult> = Vec::new();
        if let Some(aoe_damage) = weapon_aoe_damage {
            for aoe_target_id in aoe_targets {
                let mut applied_bleed: Option<i32> = None;
                let mut requires_kill = false;

                let actual_damage = {
                    let target = self.players.get_mut(&aoe_target_id).unwrap();
                    let before_life = target.life;
                    target.life = target.life.saturating_sub(aoe_damage);
                    let dealt = before_life - target.life;

                    if dealt > 0 {
                        if let Some(bleed_value) = weapon_bleed_damage {
                            if target.update_bleed_effect(bleed_value) {
                                applied_bleed = Some(bleed_value);
                            }
                        }

                        if target.life <= 0 && target.is_alive {
                            requires_kill = true;
                        }
                    }

                    dealt
                };

                if actual_damage == 0 {
                    continue;
                }

                if requires_kill {
                    let mut death_outcome =
                        self.kill_player(&aoe_target_id, Some(player_id), "攻击致死")?;
                    death_results.append(&mut death_outcome.results);
                }

                let (target_name, life_after, is_alive_after, bleed_after) = {
                    let target = self.players.get(&aoe_target_id).unwrap();
                    (
                        target.name.clone(),
                        target.life,
                        target.is_alive,
                        target.bleed_damage,
                    )
                };

                aoe_impacts.push((
                    aoe_target_id.clone(),
                    target_name.clone(),
                    actual_damage,
                    life_after,
                    is_alive_after,
                    applied_bleed,
                ));

                let mut victim_message = format!("你受到溅射攻击，损失 {} 点生命值", actual_damage);
                if let Some(bleed_value) = applied_bleed {
                    victim_message.push_str(&format!(" 并受到 {} 点流血效果", bleed_value));
                }

                let target_data = serde_json::json!({
                    "message": victim_message,
                    "life": life_after,
                    "is_alive": is_alive_after,
                    "bleed_damage": bleed_after,
                });

                aoe_results.push(ActionResult::new_system_message(
                    target_data,
                    vec![aoe_target_id],
                    victim_message,
                    false,
                ));
            }
        }

        // 获取主目标当前状态
        let (target_player_life, target_player_is_alive, target_player_bleed) = {
            let target_player = self
                .players
                .get(&target_player_id)
                .ok_or("Target player not found".to_string())?;
            (
                target_player.life,
                target_player.is_alive,
                target_player.bleed_damage,
            )
        };

        let mut attacker_formatted_message = format!(
            "{} 使用{}攻击 {} 造成 {} 点伤害",
            attacker_name, attack_method, target_player_name, main_actual_damage
        );
        if let Some(bleed_value) = main_bleed_value {
            attacker_formatted_message.push_str(&format!(" 并附加 {} 点流血", bleed_value));
        }

        if !aoe_impacts.is_empty() {
            let mut segments: Vec<String> = Vec::new();
            for impact in &aoe_impacts {
                let mut segment = format!("{} 受到 {} 点伤害", impact.1, impact.2);
                if let Some(bleed_value) = impact.5 {
                    segment.push_str(&format!(" 并流血 {}", bleed_value));
                }
                if !impact.4 {
                    segment.push_str("（阵亡）");
                }
                segments.push(segment);
            }
            attacker_formatted_message.push_str("；溅射命中 ");
            attacker_formatted_message.push_str(&segments.join("，"));
        }

        let mut victim_formatted_message =
            format!("你被攻击了，受到 {} 点伤害", main_actual_damage);
        if let Some(bleed_value) = main_bleed_value {
            victim_formatted_message.push_str(&format!(" 并受到 {} 点流血效果", bleed_value));
        }

        let aoe_hits_data: Vec<serde_json::Value> = aoe_impacts
            .iter()
            .map(|impact| {
                serde_json::json!({
                    "player_id": impact.0,
                    "player_name": impact.1,
                    "damage": impact.2,
                    "life": impact.3,
                    "is_alive": impact.4,
                    "bleed_damage": impact.5,
                })
            })
            .collect();

        // 向攻击者发送攻击结果（包括溅射与流血信息）
        let data = serde_json::json!({
            "message": attacker_formatted_message,
            "target_player_life": target_player_life,
            "target_player_is_alive": target_player_is_alive,
            "target_player_bleed_damage": target_player_bleed,
            "attack_method": attack_method,
            "damage": main_actual_damage,
            "bleed_damage": main_bleed_value,
            "aoe_hits": aoe_hits_data,
            "aoe_damage": weapon_aoe_damage,
        });

        // 向被攻击者发送通知
        let target_data = serde_json::json!({
            "message": victim_formatted_message,
            "life": target_player_life,
            "is_alive": target_player_is_alive,
            "bleed_damage": target_player_bleed,
        });

        // 消耗体力值并清除上一次搜索结果，防止连续攻击同一目标
        self.clear_player_search_result(player_id);

        self.consume_strength(player_id, attack_cost)?;

        // 创建动作结果
        let full_action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            attacker_formatted_message,
            true,
        );

        let diff_action_result = ActionResult::new_system_message(
            target_data,
            vec![target_player_id.to_string()],
            victim_formatted_message,
            false,
        );

        // 汇总所有ActionResult并返回
        let mut results = vec![full_action_result, diff_action_result];
        results.extend(aoe_results);
        if let Some(action) = weapon_destroyed_result {
            results.push(action);
        }
        if let Some(action) = armor_destroyed_result {
            results.push(action);
        }
        results.extend(death_results);

        let action_results = ActionResults { results };

        Ok(action_results)
    }
}
