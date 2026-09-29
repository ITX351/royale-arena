//! 玩家普通行动处理

use std::collections::HashMap;

use rand::Rng;

use crate::websocket::models::{
    ActionResult, ActionResults, GameState, SearchResultType, SearchTarget,
};

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
        self.record_move_for_rest_mode(player_id);
        self.clear_player_search_result(player_id);

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

        // 处理搜索结果
        match selected_target {
            SearchTarget::Player(target_player_id) => {
                use rand::Rng;
                let mut rng = rand::rng();
                let reveal_roll: f64 = rng.random_range(0.0..1.0);
                let reveal_name = reveal_roll <= self.weather;
                self.handle_player_search_result(player_id, &target_player_id, reveal_name)
            }
            SearchTarget::Item(item_id) => self.handle_item_search_result(player_id, &item_id),
        }
    }

    /// 处理捡拾行动
    pub fn handle_pick_action(&mut self, player_id: &str) -> Result<ActionResults, String> {
        // 使用规则引擎获取拾取消耗
        let pick_cost = self.rule_engine.action_costs.pick;

        // 使用规则引擎检查背包容量（使用总物品数量）
        {
            let player = self.players.get(player_id).unwrap();

            if player.get_total_item_count() >= player.max_backpack_items {
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
                }
                self.clear_player_search_result(player_id);

                // 消耗体力值
                self.consume_strength(player_id, pick_cost)?;

                // 返回更新后的玩家信息
                let data = serde_json::json!({
                    "inventory": self.players.get(player_id).unwrap().inventory,
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
            "inventory": self.players.get(player_id).unwrap().inventory,
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

    /// 商店购买处理
    pub fn handle_shop_buy_action(
        &mut self,
        player_id: &str,
        buy_items: &[crate::websocket::models::ShopBuyItem],
    ) -> Result<ActionResults, String> {
        if buy_items.is_empty() {
            let data = serde_json::json!({});
            return Ok(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "未选择任何商品".to_string(),
                false,
            )
            .as_results());
        }

        let info_message = |message: String| {
            ActionResult::new_info_message(
                serde_json::json!({}),
                vec![player_id.to_string()],
                message,
                false,
            )
            .as_results()
        };

        let mut aggregated_quantities: HashMap<String, i32> = HashMap::new();
        let mut listing_order: Vec<String> = Vec::new();
        for buy in buy_items {
            if buy.quantity < 1 {
                continue;
            }

            if let Some(existing_qty) = aggregated_quantities.get_mut(&buy.listing_id) {
                *existing_qty = match existing_qty.checked_add(buy.quantity) {
                    Some(quantity) => quantity,
                    None => {
                        return Ok(info_message("购买数量过大，交易已取消".to_string()));
                    }
                };
            } else {
                listing_order.push(buy.listing_id.clone());
                aggregated_quantities.insert(buy.listing_id.clone(), buy.quantity);
            }
        }

        // 验证并收集购买信息：(listing_id, item_name, price, buy_qty, kind_rarity)
        let mut purchase_plan: Vec<(String, String, i32, i32, Option<(String, String)>)> =
            Vec::new();
        let mut total_cost: i32 = 0;
        let mut total_items: usize = 0;

        for listing_id in listing_order {
            let buy_qty = aggregated_quantities[&listing_id];
            let listing = match self.shop.iter().find(|l| l.id == listing_id) {
                Some(l) => l.clone(),
                None => {
                    return Ok(info_message(format!(
                        "商品 {} 不存在或已被下架",
                        listing_id
                    )));
                }
            };

            if buy_qty > listing.quantity {
                return Ok(info_message(format!(
                    "商品 {} 库存不足，请求 {} 但仅剩 {}",
                    listing.item_name, buy_qty, listing.quantity
                )));
            }

            // 校验商品价格为正值，防止数据篡改导致的经济漏洞
            if listing.price < 1 {
                return Ok(info_message(format!(
                    "商品 {} 价格异常（{}），交易已取消",
                    listing.item_name, listing.price
                )));
            }

            let line_cost = match listing.price.checked_mul(buy_qty) {
                Some(cost) => cost,
                None => {
                    return Ok(info_message(format!(
                        "商品 {} 的总价计算溢出，交易已取消",
                        listing.item_name
                    )));
                }
            };
            total_cost = match total_cost.checked_add(line_cost) {
                Some(cost) => cost,
                None => {
                    return Ok(info_message("本次购买总价过大，交易已取消".to_string()));
                }
            };
            let buy_qty_usize = match usize::try_from(buy_qty) {
                Ok(quantity) => quantity,
                Err(_) => {
                    return Ok(info_message("购买数量无效，交易已取消".to_string()));
                }
            };
            total_items = match total_items.checked_add(buy_qty_usize) {
                Some(quantity) => quantity,
                None => {
                    return Ok(info_message("购买数量过大，交易已取消".to_string()));
                }
            };
            purchase_plan.push((
                listing.id.clone(),
                listing.item_name.clone(),
                listing.price,
                buy_qty,
                listing.item_kind.clone().zip(listing.rarity.clone()),
            ));
        }

        if purchase_plan.is_empty() {
            let data = serde_json::json!({});
            return Ok(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "未选择任何商品".to_string(),
                false,
            )
            .as_results());
        }

        // 检查玩家货币是否足够
        let player = self.players.get(player_id).ok_or("Player not found")?;
        if player.coins < total_cost as f64 {
            let data = serde_json::json!({});
            return Ok(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                format!("货币不足，需要 {} 但只有 {}", total_cost, player.coins),
                false,
            )
            .as_results());
        }

        // 检查背包空间
        let max_inventory_size = player.max_backpack_items;
        let current_items = player.get_total_item_count();
        if current_items + total_items > max_inventory_size {
            let data = serde_json::json!({});
            return Ok(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                format!(
                    "背包空间不足，需要 {} 个空位但只有 {} 个",
                    total_items,
                    max_inventory_size.saturating_sub(current_items)
                ),
                false,
            )
            .as_results());
        }

        // 预先创建所有物品（原子性检查），任何一个失败则中止整笔交易
        let player_name = self.players.get(player_id).unwrap().name.clone();
        let mut created_items = Vec::new();
        let existing_names = self.collect_existing_weapons_and_armor_names();
        let mut drawn_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (_id, item_name, _price, qty, kind_rarity) in &purchase_plan {
            for _ in 0..*qty {
                let item = match kind_rarity {
                    None => match self.rule_engine.create_item_from_name(item_name) {
                        Ok(item) => item,
                        Err(err) => {
                            let data = serde_json::json!({});
                            return Ok(ActionResult::new_info_message(
                                data,
                                vec![player_id.to_string()],
                                format!("创建物品 {} 失败，交易取消: {}", item_name, err),
                                false,
                            )
                            .as_results());
                        }
                    },
                    Some((kind, rarity)) => {
                        let candidates = self.rarity_display_names(kind, rarity);
                        let available: Vec<&String> = candidates
                            .iter()
                            .filter(|n| !existing_names.contains(*n) && !drawn_names.contains(*n))
                            .collect();
                        if available.is_empty() {
                            let kind_cn =
                                Self::item_kind_display_name(kind).unwrap_or(kind.as_str());
                            let rarity_cn =
                                Self::rarity_display_name(rarity).unwrap_or(rarity.as_str());
                            let data = serde_json::json!({});
                            return Ok(ActionResult::new_info_message(
                                data,
                                vec![player_id.to_string()],
                                format!("{}类{}可抽选的名称已全部在场，购买失败", rarity_cn, kind_cn),
                                false,
                            )
                            .as_results());
                        }
                        let mut rng = rand::rng();
                        let name = available[rng.random_range(0..available.len())].clone();
                        drawn_names.insert(name.clone());
                        match self.rule_engine.create_item_from_name(&name) {
                            Ok(item) => item,
                            Err(err) => {
                                let data = serde_json::json!({});
                                return Ok(ActionResult::new_info_message(
                                    data,
                                    vec![player_id.to_string()],
                                    format!("创建物品 {} 失败，交易取消: {}", name, err),
                                    false,
                                )
                                .as_results());
                            }
                        }
                    }
                };
                created_items.push(item);
            }
        }

        // 所有物品创建成功后，一次性加入背包、扣除货币、减少库存
        let player = self.players.get_mut(player_id).unwrap();
        let item_names: Vec<String> = created_items.iter().map(|i| i.name.clone()).collect();
        player.inventory.extend(created_items);

        // 扣除货币
        player.coins -= total_cost as f64;

        // 扣减库存或移除售罄商品
        for (listing_id, _, _, buy_qty, _) in &purchase_plan {
            if let Some(listing) = self.shop.iter_mut().find(|l| l.id == *listing_id) {
                listing.quantity = listing
                    .quantity
                    .checked_sub(*buy_qty)
                    .expect("validated shop purchase should not underflow listing quantity");
            }
        }
        self.shop.retain(|l| l.quantity > 0);

        let detail_data = serde_json::json!({
            "purchased_items": item_names,
            "total_cost": total_cost,
            "remaining_coins": player.coins,
        });

        let detail_result = ActionResult::new_system_message(
            detail_data,
            vec![player_id.to_string()],
            format!(
                "{} 从商店购买了 {} 件物品，花费 {} 货币",
                player_name,
                item_names.len(),
                total_cost
            ),
            true,
        );

        Ok(ActionResults {
            results: vec![detail_result],
        })
    }

    /// 处理道具转移行动（队友模式位 8）
    /// 发起方免费，接收方体力 -5
    pub fn handle_transfer_item_action(
        &mut self,
        sender_id: &str,
        item_id: &str,
        target_player_id: &str,
    ) -> Result<ActionResults, String> {
        let info_message = |message: String, sender: &str| -> ActionResults {
            ActionResult::new_info_message(
                serde_json::json!({}),
                vec![sender.to_string()],
                message,
                false,
            )
            .as_results()
        };

        // 1. 位 8 必须开
        if !self.rule_engine.teammate_behavior.is_transfer_enabled() {
            return Ok(info_message("队友物品转移未开启".to_string(), sender_id));
        }
        // 2. 必须是同队
        if !self.are_teammates(sender_id, target_player_id) {
            return Ok(info_message("目标不是你的队友".to_string(), sender_id));
        }
        // 3. 接收方必须存活
        let target_alive = self
            .players
            .get(target_player_id)
            .map(|p| p.is_alive)
            .unwrap_or(false);
        if !target_alive {
            return Ok(info_message("对方已阵亡，无法接收".to_string(), sender_id));
        }
        // 4. 物品必须在背包
        let item = {
            let sender = self.players.get(sender_id).ok_or("Sender not found")?;
            sender.inventory.iter().find(|i| i.id == item_id).cloned()
        };
        let item = match item {
            Some(it) => it,
            None => return Ok(info_message("物品不在背包中".to_string(), sender_id)),
        };
        // 5. 接收方体力 ≥ 5
        let target_strength = self
            .players
            .get(target_player_id)
            .map(|p| p.strength)
            .unwrap_or(0);
        if target_strength < 5 {
            return Ok(info_message("对方体力不足，无法接收".to_string(), sender_id));
        }
        // 6. 接收方背包未满
        let target = self.players.get(target_player_id);
        let max = target.map(|p| p.max_backpack_items).unwrap_or(0);
        let target_count = target.map(|p| p.get_total_item_count()).unwrap_or(0);
        if target_count >= max {
            return Ok(info_message("对方背包已满，无法接收".to_string(), sender_id));
        }

        // 应用：sender 移除、target 加入并扣体力
        let sender_name = self.players.get(sender_id).unwrap().name.clone();
        let target_name = self.players.get(target_player_id).unwrap().name.clone();
        let item_name = item.name.clone();

        self.players
            .get_mut(sender_id)
            .unwrap()
            .inventory
            .retain(|i| i.id != item_id);
        let target = self.players.get_mut(target_player_id).unwrap();
        target.inventory.push(item);
        target.strength -= 5;
        let target_strength_after = target.strength;
        let target_life = target.life;

        let sender_msg = format!("你将 {} 转移给了 {}", item_name, target_name);
        let target_msg = format!("队友 {} 将 {} 转移给你，体力 -5", sender_name, item_name);
        let director_msg = format!(
            "玩家 {} 给玩家 {} 转移了物品 {}（接收方体力 -5）",
            sender_name, target_name, item_name
        );

        let sender_data = serde_json::json!({
            "item_name": &item_name,
            "target": &target_name,
        });
        let target_data = serde_json::json!({
            "item_name": &item_name,
            "sender": &sender_name,
            "strength": target_strength_after,
            "life": target_life,
        });
        let director_data = serde_json::json!({
            "item_name": item_name,
            "sender": sender_name,
            "target": target_name,
        });

        Ok(ActionResults {
            results: vec![
                ActionResult::new_system_message(
                    sender_data,
                    vec![sender_id.to_string()],
                    sender_msg,
                    false,
                ),
                ActionResult::new_system_message(
                    target_data,
                    vec![target_player_id.to_string()],
                    target_msg,
                    false,
                ),
                ActionResult::new_system_message(
                    director_data,
                    vec![],
                    director_msg,
                    true,
                ),
            ],
        })
    }

    /// 处理售出行动：白天按稀有度价格出售背包中的武器/防具（绿色需成对）
    pub fn handle_sell_item_action(
        &mut self,
        player_id: &str,
        item_ids: &[String],
    ) -> Result<ActionResults, String> {
        let info_message = |message: String, sender: &str| -> ActionResults {
            ActionResult::new_info_message(
                serde_json::json!({}),
                vec![sender.to_string()],
                message,
                false,
            )
            .as_results()
        };

        // 1. 时间窗：夜窗已设置且当前不在夜间才可售出
        match (self.night_start_time, self.night_end_time) {
            (Some(start_time), Some(end_time)) => {
                let now = chrono::Utc::now();
                if now >= start_time && now <= end_time {
                    return Ok(info_message(
                        "售出只在非夜间行动时间可用".to_string(),
                        player_id,
                    ));
                }
            }
            _ => {
                return Ok(info_message(
                    "导演尚未设置夜晚行动时间，无法售出".to_string(),
                    player_id,
                ));
            }
        }

        // 2. 数量：1 件非绿色或 2 件绿色
        if item_ids.is_empty() || item_ids.len() > 2 {
            return Ok(info_message(
                "一次只能售出 1 件非绿色物品或 2 件绿色物品".to_string(),
                player_id,
            ));
        }

        if item_ids.len() == 2 && item_ids[0] == item_ids[1] {
            return Ok(info_message(
                "不能重复选择同一件物品".to_string(),
                player_id,
            ));
        }

        // 3. 逐件校验：在背包、武器/防具、稀有度有价
        let mut items = Vec::new();
        for id in item_ids {
            let Some(item) = self
                .players
                .get(player_id)
                .ok_or("Player not found")?
                .inventory
                .iter()
                .find(|i| i.id == *id)
                .cloned()
            else {
                return Ok(info_message(format!("物品 {} 不在背包中", id), player_id));
            };
            if !crate::game::game_rule_engine::Item::is_weapon_or_armor(&item) {
                return Ok(info_message("只有武器和防具可以售出".to_string(), player_id));
            }
            let Some(rarity) = item.rarity.as_deref() else {
                return Ok(info_message(
                    "该道具稀有度未开放售出".to_string(),
                    player_id,
                ));
            };
            if !self.sell_prices.iter().any(|e| e.rarity == rarity) {
                return Ok(info_message(
                    "该道具稀有度未开放售出".to_string(),
                    player_id,
                ));
            }
            items.push(item);
        }

        // 4. 配对约束：1 件不得为绿；2 件必须全绿
        let is_green =
            |i: &crate::game::game_rule_engine::Item| i.rarity.as_deref() == Some("common");
        if items.len() == 1 && is_green(&items[0]) {
            return Ok(info_message("绿色物品需成对售出".to_string(), player_id));
        }
        if items.len() == 2 && !items.iter().all(|i| is_green(i)) {
            return Ok(info_message(
                "绿色物品不能与其他稀有度混合售出".to_string(),
                player_id,
            ));
        }

        // 5. 原子应用：移除全部、累计入账
        let total_price: f64 = items
            .iter()
            .map(|i| {
                let rarity = i.rarity.as_deref().unwrap();
                self.sell_prices
                    .iter()
                    .find(|e| e.rarity == rarity)
                    .unwrap()
                    .price
            })
            .sum();
        let player_name = self.players.get(player_id).unwrap().name.clone();
        let item_names = items.iter().map(|i| i.name.clone()).collect::<Vec<_>>();
        let names_str = item_names.join("、");
        let rarity = items[0].rarity.clone().unwrap();
        let rarity_display = GameState::rarity_display_name(&rarity)
            .unwrap_or(rarity.as_str())
            .to_string();

        {
            let player = self.players.get_mut(player_id).unwrap();
            player.inventory.retain(|i| !item_ids.contains(&i.id));
            player.coins += total_price;
        }
        let coins_after = self.players.get(player_id).unwrap().coins;

        let price_str = GameState::format_price(total_price);
        let seller_msg = format!("你售出了 {}，获得 {} 货币", names_str, price_str);
        let director_msg = format!(
            "玩家 {} 售出了 {}（{}类），获得 {} 货币",
            player_name, names_str, rarity_display, price_str
        );

        let seller_data = serde_json::json!({
            "item_names": &item_names,
            "price": total_price,
            "coins": coins_after,
        });
        let director_data = serde_json::json!({
            "item_names": item_names,
            "player": player_name,
            "items": items
                .iter()
                .map(|i| serde_json::json!({ "name": &i.name, "rarity": &i.rarity }))
                .collect::<Vec<_>>(),
            "price": total_price,
        });

        Ok(ActionResults {
            results: vec![
                ActionResult::new_system_message(
                    seller_data,
                    vec![player_id.to_string()],
                    seller_msg,
                    false,
                ),
                ActionResult::new_system_message(director_data, vec![], director_msg, true),
            ],
        })
    }
}
