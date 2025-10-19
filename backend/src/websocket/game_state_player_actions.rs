//! GameState 玩家行动实现

use super::models::*;

impl GameState {
    /// 消耗玩家体力值
    ///
    /// # 参数
    /// - `player_id`: 玩家ID
    /// - `amount`: 消耗的体力值
    ///
    /// # 返回值
    /// - `Ok(())`: 体力消耗成功
    /// - `Err(String)`: 玩家未找到
    /// 处理玩家出生行动
    pub fn handle_born_action(
        &mut self,
        player_id: &str,
        place_name: &str,
    ) -> Result<ActionResults, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).unwrap();

        // 验证指定地点是否存在且未被摧毁
        let place = self
            .places
            .get(place_name)
            .ok_or("Place not found".to_string())?;
        if place.is_destroyed {
            let action_result = ActionResult::new_info_message(
                serde_json::json!({}),
                vec![player_id.to_string()],
                "地点已被摧毁".to_string(),
                false,
            );
            return Ok(action_result.as_results());
        }

        // 更新玩家位置到指定地点
        player.location = place_name.to_string();

        // 将玩家添加到地点的玩家列表中
        let place_mut = self.places.get_mut(place_name).unwrap();
        place_mut.players.push(player.id.clone());

        // 向该玩家发送位置更新结果
        let data = serde_json::json!({
            "location": place_name
        });

        // 创建动作结果，只广播给发起者本人
        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!("{} 在地点 {} 出生", player.name, place_name),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 处理玩家移动行动
    pub fn handle_move_action(
        &mut self,
        player_id: &str,
        target_place: &str,
    ) -> Result<ActionResults, String> {
        // 使用规则引擎获取移动消耗
        let move_cost = self.rule_engine.action_costs.move_cost;

        // 验证目标地点是否存在且未被摧毁
        let place = self
            .places
            .get(target_place)
            .ok_or("Target place not found".to_string())?;
        if place.is_destroyed {
            // 用Info类型返回错误提示
            let action_result = ActionResult::new_info_message(
                serde_json::json!({}),
                vec![player_id.to_string()],
                "目标地点已被摧毁".to_string(),
                false,
            );
            return Ok(action_result.as_results());
        }

        // 获取玩家引用并移动玩家到目标位置
        let player = self.players.get_mut(player_id).unwrap();
        let player_location = player.location.clone();
        let player_name = player.name.clone();
        player.location = target_place.to_string();

        // 从当前地点移除玩家
        if let Some(current_place) = self.places.get_mut(&player_location) {
            current_place.players.retain(|id| id != player_id);
        }

        // 将玩家添加到目标地点的玩家列表中
        if let Some(target_place_obj) = self.places.get_mut(target_place) {
            target_place_obj.players.push(player_id.to_string());
        }

        // 消耗体力值
        self.consume_strength(player_id, move_cost)?;

        // 向该玩家发送位置更新结果
        let data = serde_json::json!({
            "location": target_place,
            "strength": self.players.get(player_id).unwrap().strength
        });

        // 创建动作结果，只广播给发起者本人
        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!("{} 移动到地点 {}", player_name, target_place),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 处理搜索行动（优化版本）
    pub fn handle_search_action(&mut self, player_id: &str) -> Result<ActionResults, String> {
        // 使用规则引擎获取搜索消耗
        let search_cost = self.rule_engine.action_costs.search;

        // 获取玩家状态信息（避免借用冲突）
        let player_last_search_time = self.players.get(player_id).unwrap().last_search_time;

        // 使用规则引擎获取搜索冷却时间
        let search_cooldown = self.rule_engine.get_search_cooldown();
        if let Some(last_search_time) = player_last_search_time {
            let elapsed = chrono::Utc::now().signed_duration_since(last_search_time);
            if elapsed.num_seconds() < search_cooldown {
                let remaining_time = search_cooldown - elapsed.num_seconds();
                let data = serde_json::json!({});
                let action_result = ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    format!("搜索冷却中，请等待{}秒后再试", remaining_time),
                    false,
                );
                return Ok(action_result.as_results());
            }
        }

        // 更新玩家状态
        {
            let player = self.players.get_mut(player_id).unwrap();
            player.last_search_time = Some(chrono::Utc::now());
        }

        // 消耗体力值
        self.consume_strength(player_id, search_cost)?;

        // 汇总当前地点的所有搜索目标
        let search_targets = self.collect_search_targets(player_id);

        if search_targets.is_empty() {
            // 没有搜索目标，返回空结果
            return self.handle_empty_search_result(player_id);
        }

        // 等概率随机选择一个目标
        let selected_target = self.select_random_target(&search_targets);

        // 根据天气条件确定结果可见性
        use rand::Rng;
        let mut rng = rand::rng();
        let is_visible = rng.random_bool(self.weather);

        // 处理搜索结果
        match selected_target {
            SearchTarget::Player(target_player_id) => {
                self.handle_player_search_result(player_id, &target_player_id, is_visible)
            }
            SearchTarget::Item(item_id) => {
                self.handle_item_search_result(player_id, &item_id, is_visible)
            }
        }
    }

    /// 处理捡拾行动
    pub fn handle_pick_action(&mut self, player_id: &str) -> Result<ActionResults, String> {
        // 使用规则引擎获取拾取消耗
        let pick_cost = self.rule_engine.action_costs.pick;

        // 使用规则引擎检查背包容量（使用总物品数量）
        {
            let player = self.players.get(player_id).unwrap();
            let max_backpack_items = self.rule_engine.player_config.max_backpack_items as usize;

            if player.get_total_item_count() >= max_backpack_items {
                // 背包已满，返回Info提示
                let action_result = ActionResult::new_info_message(
                    serde_json::json!({}),
                    vec![player_id.to_string()],
                    format!("{} 尝试拾取物品但背包已满", player.name),
                    false, // 不向导演广播
                );
                return Ok(action_result.as_results());
            }
        }

        // 检查上一次搜索结果是否为物品
        {
            let player = self.players.get(player_id).unwrap();

            let last_search_result_valid =
                if let Some(ref search_result) = player.last_search_result {
                    search_result.target_type == SearchResultType::Item
                } else {
                    false
                };

            if !last_search_result_valid {
                // 用Info类型返回错误提示
                let action_result = ActionResult::new_info_message(
                    serde_json::json!({}),
                    vec![player_id.to_string()],
                    "上一次搜索结果不是物品".to_string(),
                    false,
                );
                return Ok(action_result.as_results());
            }
        }

        // 获取搜索结果信息和玩家位置
        let (player_last_search_result, player_location) = {
            let player = self.players.get(player_id).unwrap();
            (player.last_search_result.clone(), player.location.clone())
        };

        let item_id = if let Some(ref search_result) = player_last_search_result {
            search_result.target_id.clone()
        } else {
            return Err("No previous search result".to_string());
        };

        // 验证上一次搜索到的物品是否仍然存在
        if let Some(place) = self.places.get_mut(&player_location) {
            let item_index = place.items.iter().position(|item| item.id == item_id);
            if let Some(item_index) = item_index {
                // 从地点物品列表中移除物品
                let item = place.items.remove(item_index);
                let item_name = item.name.clone();

                // 将物品添加到玩家背包并清除上一次搜索结果
                {
                    let player = self.players.get_mut(player_id).unwrap();
                    player.inventory.push(item);
                    // 清除捡拾者的上一次搜索结果，防止连续捡拾同一物品
                    player.last_search_result = None;
                }

                // 消耗体力值
                self.consume_strength(player_id, pick_cost)?;

                // 返回更新后的玩家信息
                let data = serde_json::json!({
                    "inventory": self.players.get(player_id).unwrap().inventory.clone(),
                    "strength": self.players.get(player_id).unwrap().strength
                });

                // 创建动作结果，只广播给发起者本人
                let action_result = ActionResult::new_system_message(
                    data,
                    vec![player_id.to_string()],
                    format!(
                        "{} 捡起了一个物品 {}",
                        self.players.get(player_id).unwrap().name,
                        item_name
                    ),
                    true,
                );
                Ok(action_result.as_results())
            } else {
                // 物品不存在，向该玩家发送捡拾失败消息
                let action_result = ActionResult::new_system_message(
                    serde_json::json!({}),
                    vec![player_id.to_string()],
                    format!(
                        "{} 试图捡起一个物品但该物品已不存在",
                        self.players.get(player_id).unwrap().name
                    ),
                    true,
                );
                Ok(action_result.as_results())
            }
        } else {
            Err("Player location not found".to_string())
        }
    }

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

        // 验证目标玩家是否存在且在同一地点
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
        let failed_message = "目标玩家已离开该地点".to_string();
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

        // 根据防具减免伤害
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
                    target_player.set_bleed_effect(bleed_value, 0);
                    main_bleed_value = Some(bleed_value);
                }
                if target_player.life == 0 && target_player.is_alive {
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
                            target.set_bleed_effect(bleed_value, 0);
                            applied_bleed = Some(bleed_value);
                        }

                        if target.life == 0 && target.is_alive {
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
                        self.kill_player(&aoe_target_id, Some(player_id), "溅射伤害致死")?;
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

                let mut victim_message = format!(
                    "你受到溅射攻击，损失 {} 点生命值",
                    actual_damage
                );
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
        {
            let player = self.players.get_mut(player_id).unwrap();
            player.last_search_result = None;
        }

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

    /// 处理装备行动
    pub fn handle_equip_action(
        &mut self,
        player_id: &str,
        item_id: &str,
    ) -> Result<ActionResults, String> {
        // 使用规则引擎获取装备消耗
        let equip_cost = self.rule_engine.action_costs.equip;

        let item_index = {
            let player = self.players.get(player_id).unwrap();
            player.inventory.iter().position(|item| item.id == item_id)
        };
        if item_index.is_none() {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "背包中没有该道具".to_string(),
                false,
            );
            return Ok(action_result.as_results());
        }

        let item_index = item_index.unwrap();

        let (item_snapshot, player_name) = {
            let player_view = self.players.get(player_id).unwrap();
            (
                player_view.inventory[item_index].clone(),
                player_view.name.clone(),
            )
        };

        let item_name = item_snapshot.name.clone();
        let is_weapon = item_snapshot.as_weapon().is_some();
        let is_armor = item_snapshot.as_armor().is_some();

        if is_weapon {
            let (equipped_weapon, inventory) = {
                let player = self.players.get_mut(player_id).unwrap();
                let item = player.inventory.remove(item_index);

                if let Some(old_weapon) = player.equip_weapon(item) {
                    player.inventory.push(old_weapon);
                }

                (player.equipped_weapon.clone(), player.inventory.clone())
            };

            self.consume_strength(player_id, equip_cost)?;

            let data = serde_json::json!({
                "equipped_weapon": equipped_weapon,
                "inventory": inventory,
                "strength": self.players.get(player_id).unwrap().strength
            });

            let action_result = ActionResult::new_system_message(
                data,
                vec![player_id.to_string()],
                format!("{} 装备了武器 {}", player_name, item_name),
                true,
            );
            Ok(action_result.as_results())
        } else if is_armor {
            let (equipped_armor, inventory) = {
                let player = self.players.get_mut(player_id).unwrap();
                let item = player.inventory.remove(item_index);

                if let Some(old_armor) = player.equip_armor(item) {
                    player.inventory.push(old_armor);
                }

                (player.equipped_armor.clone(), player.inventory.clone())
            };

            self.consume_strength(player_id, equip_cost)?;

            let data = serde_json::json!({
                "equipped_armor": equipped_armor,
                "inventory": inventory,
                "strength": self.players.get(player_id).unwrap().strength
            });

            let action_result = ActionResult::new_system_message(
                data,
                vec![player_id.to_string()],
                format!("{} 装备了防具 {}", player_name, item_name),
                true,
            );
            Ok(action_result.as_results())
        } else {
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "该物品不是装备，无法装备".to_string(),
                false,
            );
            Ok(action_result.as_results())
        }
    }

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

                let life_delta = (life_after - life_before).max(0);

                // 消耗体力值
                self.consume_strength(player_id, use_cost)?;

                let strength_after_use = self.players.get(player_id).unwrap().strength;

                let mut log_message = format!(
                    "{} 使用了 {}，生命值{}({:+})，体力{}",
                    player_name, item_name, life_after, life_delta, strength_after_use
                );
                if curing_bleed {
                    log_message.push_str("，解除了流血");
                }

                let data = serde_json::json!({
                    "life": life_after,
                    "bleed_damage": bleed_damage,
                    "strength": strength_after_use
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
                let restored_amount = {
                    let player = self.players.get_mut(player_id).unwrap();
                    let before = player.strength;
                    player.strength += effect.effect_value;
                    if player.strength > player.max_strength {
                        player.strength = player.max_strength;
                    }
                    let after = player.strength;
                    (after - before).max(0)
                };

                // 消耗使用体力（在恢复后扣除）
                self.consume_strength(player_id, use_cost)?;

                let strength_after_use = self.players.get(player_id).unwrap().strength;

                let log_message = format!(
                    "{} 使用了 {}，体力{}(+{})",
                    player_name, item_name, strength_after_use, restored_amount
                );

                let data = serde_json::json!({
                    "strength": strength_after_use,
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

    /// 处理丢弃道具行动
    pub fn handle_throw_action(
        &mut self,
        player_id: &str,
        item_id: &str,
    ) -> Result<ActionResults, String> {
        // 使用规则引擎获取丢弃消耗
        let throw_cost = self.rule_engine.action_costs.throw_item;

        // 获取玩家引用
        let player = self.players.get_mut(player_id).unwrap();

        // 验证玩家背包中是否存在指定物品
        let item_name =
            if let Some(item_index) = player.inventory.iter().position(|item| item.id == item_id) {
                // 从玩家背包中移除物品
                let item = player.inventory.remove(item_index);
                let item_name = item.name.clone();
                let player_location = player.location.clone();

                // 将物品添加到当前地点的物品列表
                if let Some(place) = self.places.get_mut(&player_location) {
                    place.items.push(item);
                }
                item_name
            } else {
                // 用Info类型返回错误提示
                let data = serde_json::json!({});
                let action_result = ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    "背包中没有该道具".to_string(),
                    false,
                );
                return Ok(action_result.as_results());
            }; // 释放player借用

        // 消耗体力值
        self.consume_strength(player_id, throw_cost)?;

        // 向该玩家发送背包更新
        let data = serde_json::json!({
            "inventory": self.players.get(player_id).unwrap().inventory.clone(),
            "strength": self.players.get(player_id).unwrap().strength
        });

        // 创建动作结果，只广播给发起者本人
        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "{} 丢弃了物品 {}",
                self.players.get(player_id).unwrap().name,
                item_name
            ),
            true,
        );
        Ok(action_result.as_results())
    }

    /// 处理传音行动
    pub fn handle_deliver_action(
        &mut self,
        player_id: &str,
        target_player_id: &str,
        message: &str,
    ) -> Result<ActionResults, String> {
        // 使用规则引擎获取传音消耗
        let deliver_cost = self.rule_engine.action_costs.deliver;

        // 获取发送玩家和目标玩家信息
        let target_player_name = self
            .players
            .get(target_player_id)
            .ok_or("Target player not found".to_string())?
            .name
            .clone();
        let sender_player_name = self.players.get(player_id).unwrap().name.clone();

        // 消耗体力值
        self.consume_strength(player_id, deliver_cost)?;

        // 向发送者和导演发送完整消息
        let sender_formatted_message = format!(
            "{} 向 {} 发送消息: {}",
            sender_player_name, target_player_name, message
        );

        let sender_data = serde_json::json!({
            "message": sender_formatted_message,
        });

        // 向接收者发送差分消息
        let receiver_formatted_message =
            format!("你收到了来自 {} 的消息: {}", sender_player_name, message);

        let receiver_data = serde_json::json!({
            "message": receiver_formatted_message,
        });

        // 创建动作结果，向发送者和导演发送完整消息
        let sender_action_result = ActionResult::new_user_message(
            sender_data,
            vec![player_id.to_string()],
            sender_formatted_message,
            true, // 向导演广播
        );

        // 创建动作结果，向接收者发送差分消息（不向导演广播）
        let receiver_action_result = ActionResult::new_user_message(
            receiver_data,
            vec![target_player_id.to_string()],
            receiver_formatted_message,
            false, // 不向导演广播
        );

        // 将两个ActionResult打包成ActionResults返回
        let action_results = ActionResults {
            results: vec![sender_action_result, receiver_action_result],
        };

        Ok(action_results)
    }

    /// 处理发送消息给导演行动
    pub fn handle_send_to_director_action(
        &mut self,
        player_id: &str,
        message: &str,
    ) -> Result<ActionResults, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).unwrap();

        // 将消息转发给导演客户端
        let sender_formatted_message = format!("{} 向导演发送消息: {}", player.name, message);
        let data = serde_json::json!({
            "message": sender_formatted_message,
        });

        // 创建动作结果，只广播给发起者本人（导演会收到所有消息）
        let action_result = ActionResult::new_user_message(
            data,
            vec![player_id.to_string()],
            sender_formatted_message,
            true,
        );
        Ok(action_result.as_results())
    }

    /// 处理卸下装备行动
    pub fn handle_unequip_action(
        &mut self,
        player_id: &str,
        slot_type: &str,
    ) -> Result<ActionResults, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).unwrap();
        let player_name = player.name.clone();

        // 根据槽位类型卸下装备
        match slot_type {
            "weapon" => {
                if let Some(weapon) = player.unequip_weapon() {
                    let weapon_name = weapon.name.clone();
                    player.inventory.push(weapon);

                    let data = serde_json::json!({
                        "equipped_weapon": player.equipped_weapon,
                        "inventory": player.inventory
                    });

                    let action_result = ActionResult::new_system_message(
                        data,
                        vec![player_id.to_string()],
                        format!("{} 卸下了武器 {}", player_name, weapon_name),
                        true,
                    );
                    Ok(action_result.as_results())
                } else {
                    let data = serde_json::json!({});
                    let action_result = ActionResult::new_info_message(
                        data,
                        vec![player_id.to_string()],
                        "当前未装备武器".to_string(),
                        false,
                    );
                    Ok(action_result.as_results())
                }
            }
            "armor" => {
                if let Some(armor) = player.unequip_armor() {
                    let armor_name = armor.name.clone();
                    player.inventory.push(armor);

                    let data = serde_json::json!({
                        "equipped_armor": player.equipped_armor,
                        "inventory": player.inventory
                    });

                    let action_result = ActionResult::new_system_message(
                        data,
                        vec![player_id.to_string()],
                        format!("{} 卸下了防具 {}", player_name, armor_name),
                        true,
                    );
                    Ok(action_result.as_results())
                } else {
                    let data = serde_json::json!({});
                    let action_result = ActionResult::new_info_message(
                        data,
                        vec![player_id.to_string()],
                        "当前未装备防具".to_string(),
                        false,
                    );
                    Ok(action_result.as_results())
                }
            }
            _ => {
                let data = serde_json::json!({});
                let action_result = ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    "无效的装备槽位类型".to_string(),
                    false,
                );
                Ok(action_result.as_results())
            }
        }
    }
}
