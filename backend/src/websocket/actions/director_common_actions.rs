//! GameState 导演控制实现

use crate::websocket::actions::utils::format_delta;
use crate::websocket::models::{
    ActionResult, ActionResults, AirdropItem, GameState, ItemDeletionItem,
};

impl GameState {
    // ===== 导演行为处理方法 =====

    /// 设置夜晚开始时间
    pub fn handle_set_night_start_time(
        &mut self,
        timestamp: &str,
    ) -> Result<ActionResults, String> {
        // 解析时间
        let time = chrono::DateTime::parse_from_rfc3339(timestamp)
            .map_err(|_| "Invalid timestamp format".to_string())?
            .with_timezone(&chrono::Utc);

        // 更新游戏状态中的夜晚开始时间
        self.night_start_time = Some(time);

        // 构造响应数据
        let data = serde_json::json!({
            "night_start_time": time
        });

        // 创建动作结果，只广播给导演（不需要通知玩家）
        let action_result = ActionResult::new_system_message(
            data,
            vec![],
            format!("导演设置夜晚开始时间至 {}", timestamp),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 设置夜晚结束时间
    pub fn handle_set_night_end_time(&mut self, timestamp: &str) -> Result<ActionResults, String> {
        // 解析时间
        let time = chrono::DateTime::parse_from_rfc3339(timestamp)
            .map_err(|_| "Invalid timestamp format".to_string())?
            .with_timezone(&chrono::Utc);

        // 更新游戏状态中的夜晚结束时间
        self.night_end_time = Some(time);

        // 构造响应数据
        let data = serde_json::json!({
            "night_end_time": time
        });

        // 创建动作结果，只广播给导演（不需要通知玩家）
        let action_result = ActionResult::new_system_message(
            data,
            vec![],
            format!("导演设置夜晚结束时间至 {}", timestamp),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 设置缩圈地点
    pub fn handle_set_destroy_places(
        &mut self,
        places: &[serde_json::Value],
    ) -> Result<ActionResults, String> {
        // 先收集并验证所有传入的地点名（要求：字符串、存在于地图且未被摧毁）
        let mut requested: Vec<String> = Vec::new();
        for v in places {
            if let Some(s) = v.as_str() {
                requested.push(s.to_string());
            }
        }

        // 找出不存在或已被摧毁的地点
        let mut problematic: Vec<String> = Vec::new();
        for name in &requested {
            match self.places.get(name) {
                Some(place) => {
                    if place.is_destroyed {
                        problematic.push(format!("{} (已被摧毁)", name));
                    }
                }
                None => {
                    problematic.push(format!("{} (地点不存在)", name));
                }
            }
        }

        if !problematic.is_empty() {
            // 如果有任一地点被摧毁或不存在，拒绝本次设置请求并以 Info 形式返回
            let log_message = format!(
                "拒绝设置缩圈地点：以下地点已被摧毁或不存在：{:?}",
                problematic
            );

            let data = serde_json::json!({
                "message_type": "Info",
                "log_message": log_message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "invalid_places": problematic
            });

            return Ok(
                ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
            );
        }

        // 全部通过，更新下一夜晚缩圈地点集合
        self.next_night_destroyed_places = requested;

        // 构造响应数据
        let data = serde_json::json!({
            "next_night_destroyed_places": self.next_night_destroyed_places
        });

        // 创建动作结果，只广播给导演（不需要通知玩家）
        let action_result = ActionResult::new_system_message(
            data,
            vec![],
            format!(
                "导演设置下一夜晚缩圈地点: {:?}",
                self.next_night_destroyed_places
            ),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 批量空投处理 - 根据规则JSON智能查找物品属性
    pub fn handle_batch_airdrop(
        &mut self,
        airdrops: Vec<AirdropItem>,
    ) -> Result<ActionResults, String> {
        // 验证所有空投物品名称是否已存在于场上
        for airdrop in &airdrops {
            if self.check_item_name_exists(&airdrop.item_name) {
                let data = serde_json::json!({
                    "message_type": "Info",
                    "log_message": format!("批量空投被拒绝: 物品 {} 已存在于场上", airdrop.item_name),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                let log_message =
                    format!("批量空投被拒绝: 物品 {} 已存在于场上", airdrop.item_name);

                return Ok(
                    ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
                );
            }
        }

        let mut success_count = 0;

        for airdrop in airdrops {
            // 根据物品名称从规则JSON中查找并创建物品
            match self.rule_engine.create_item_from_name(&airdrop.item_name) {
                Ok(item) => {
                    // 添加到指定地点
                    if let Some(place) = self.places.get_mut(&airdrop.place_name) {
                        place.items.push(item);
                        success_count += 1;
                    }
                }
                Err(err) => {
                    // 如果物品创建失败，记录但继续处理其他物品
                    eprintln!("创建物品失败: {}, 错误: {}", airdrop.item_name, err);
                }
            }
        }

        let response_data = serde_json::json!({
            "action_result": {
                "message_type": "Info",
                "log_message": format!("导演执行批量空投操作，成功 {} 项", success_count),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        let log_message = format!("导演执行批量空投操作，成功 {} 项", success_count);

        Ok(ActionResult::new_info_message(response_data, vec![], log_message, true).as_results())
    }

    /// 批量物品删除处理
    pub fn handle_batch_item_deletion(
        &mut self,
        deletions: Vec<ItemDeletionItem>,
        clear_all: bool,
    ) -> Result<ActionResults, String> {
        let mut deleted_items = Vec::new();
        let mut failed_items = Vec::new();

        if clear_all {
            // 清空全场所有物品
            for (place_name, place) in self.places.iter_mut() {
                for item in place.items.drain(..) {
                    deleted_items.push(serde_json::json!({
                        "item_name": item.name,
                        "place_name": place_name
                    }));
                }
            }
        } else {
            // 根据删除列表逐项删除
            for deletion in deletions {
                let place_name = &deletion.place_name;

                // 检查地点是否存在
                if let Some(place) = self.places.get_mut(place_name) {
                    if let Some(item_name) = &deletion.item_name {
                        // 删除指定物品
                        if let Some(pos) =
                            place.items.iter().position(|item| &item.name == item_name)
                        {
                            let item = place.items.remove(pos);
                            deleted_items.push(serde_json::json!({
                                "item_name": item.name,
                                "place_name": place_name
                            }));
                        } else {
                            // 物品不存在
                            failed_items.push(serde_json::json!({
                                "item_name": item_name,
                                "place_name": place_name,
                                "reason": "物品不存在"
                            }));
                        }
                    } else {
                        // 清空地点所有物品
                        for item in place.items.drain(..) {
                            deleted_items.push(serde_json::json!({
                                "item_name": item.name,
                                "place_name": place_name
                            }));
                        }
                    }
                } else {
                    // 地点不存在
                    let item_name_str = deletion.item_name.as_deref().unwrap_or("所有物品");
                    failed_items.push(serde_json::json!({
                        "item_name": item_name_str,
                        "place_name": place_name,
                        "reason": "地点不存在"
                    }));
                }
            }
        }

        let deleted_count = deleted_items.len();
        let failed_count = failed_items.len();

        let response_data = serde_json::json!({
            "action_result": {
                "message_type": "Info",
                "log_message": format!("导演删除物品操作完成：成功删除{}个物品，失败{}个物品", deleted_count, failed_count),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "data": {
                    "deleted_items": deleted_items,
                    "failed_items": failed_items
                }
            }
        });

        let log_message = format!(
            "导演删除物品操作完成：成功删除{}个物品，失败{}个物品",
            deleted_count, failed_count
        );

        Ok(ActionResult::new_info_message(response_data, vec![], log_message, true).as_results())
    }

    /// 调整天气
    pub fn handle_weather(&mut self, weather: f64) -> Result<ActionResults, String> {
        // 更新天气条件值
        self.weather = weather;

        // 构造响应数据
        let data = serde_json::json!({
            "weather": weather
        });

        // 创建动作结果，只广播给导演（不需要通知玩家）
        let action_result = ActionResult::new_system_message(
            data,
            vec![],
            format!("导演调整天气至 {}", weather),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 设置玩家生命值
    pub fn handle_set_player_life(
        &mut self,
        player_id: &str,
        life: i32,
    ) -> Result<ActionResults, String> {
        let (current_life, is_alive, player_name) = {
            let player = self.players.get(player_id).ok_or("Player not found")?;
            (player.life, player.is_alive, player.name.clone())
        };

        if current_life == life {
            let data = serde_json::json!({
                "player_id": player_id,
                "life": current_life,
                "is_alive": is_alive,
                "message": "生命值未发生变化"
            });
            let log_message = format!(
                "导演尝试设置 {} 生命值为 {}，但未发生变化",
                player_name, life
            );
            return Ok(
                ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
            );
        }

        if current_life <= 0 && !is_alive && life > 0 {
            return self.revive_player(player_id, life);
        }

        if current_life > 0 && life <= 0 {
            return self.kill_player(player_id, None, "导演击杀");
        }

        let mut target_life = life;
        if target_life < 0 {
            target_life = 0;
        }

        let life_change = target_life - current_life;

        let player = self.players.get_mut(player_id).unwrap();
        player.life = target_life;

        let data = serde_json::json!({
            "player_id": player_id,
            "life": player.life,
            "is_alive": player.is_alive
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "导演设置 {} 生命值为 {} ({})",
                player_name,
                target_life,
                format_delta(life_change)
            ),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 设置玩家体力值
    pub fn handle_set_player_strength(
        &mut self,
        player_id: &str,
        strength: i32,
    ) -> Result<ActionResults, String> {
        let (player_name, final_strength, strength_change) = {
            let player = self.players.get_mut(player_id).ok_or("Player not found")?;
            let target_strength = strength.max(0);

            if player.strength == target_strength {
                let data = serde_json::json!({
                    "player_id": player_id,
                    "strength": player.strength,
                    "message": "体力值未发生变化"
                });

                let log_message = format!(
                    "导演尝试设置 {} 体力值为 {}，但未发生变化",
                    player.name, target_strength
                );

                return Ok(
                    ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
                );
            }

            let player_name = player.name.clone();
            let previous_strength = player.strength;
            player.strength = target_strength;
            let final_strength = player.strength;
            let strength_change = final_strength - previous_strength;

            (player_name, final_strength, strength_change)
        };

        let data = serde_json::json!({
            "player_id": player_id,
            "strength": final_strength
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "导演设置 {} 体力值为 {} ({})",
                player_name,
                final_strength,
                format_delta(strength_change)
            ),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 向玩家添加物品
    pub fn handle_add_player_item(
        &mut self,
        player_id: &str,
        item_name: &str,
    ) -> Result<ActionResults, String> {
        // 验证物品名称是否已存在于场上
        if self.check_item_name_exists(item_name) {
            let data = serde_json::json!({
                "message_type": "Info",
                "log_message": format!("无法添加物品 {}: 该物品已存在于场上", item_name),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            let log_message = format!("无法添加物品 {}: 该物品已存在于场上", item_name);

            return Ok(
                ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
            );
        }

        // 根据物品名称从规则JSON中查找并创建物品
        let item = self
            .rule_engine
            .create_item_from_name(item_name)
            .map_err(|err| format!("创建物品失败: {}, 错误: {}", item_name, err))?;

        // 获取玩家
        let player = self
            .players
            .get_mut(player_id)
            .ok_or("Player not found".to_string())?;

        // 添加物品到玩家背包
        player.inventory.push(item.clone());

        // 构造响应数据
        let data = serde_json::json!({
            "player_id": player_id,
            "item": item,
            "action": "add"
        });

        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!("导演向 {} 添加了物品 {}", player.name, item_name),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 从玩家移除物品
    pub fn handle_remove_player_item(
        &mut self,
        player_id: &str,
        item_name: &str,
    ) -> Result<ActionResults, String> {
        // 获取玩家
        let player = self
            .players
            .get_mut(player_id)
            .ok_or("Player not found".to_string())?;

        // 优先检查已装备的物品并移除，以确保释放其占用的内存
        let mut removed_item = if player
            .equipped_weapon
            .as_ref()
            .map_or(false, |item| item.name == item_name)
        {
            player.equipped_weapon.take()
        } else if player
            .equipped_armor
            .as_ref()
            .map_or(false, |item| item.name == item_name)
        {
            player.equipped_armor.take()
        } else {
            None
        };

        if removed_item.is_none() {
            if let Some(item_pos) = player.inventory.iter().position(|i| i.name == item_name) {
                removed_item = Some(player.inventory.remove(item_pos));
            }
        }

        let removed_item = removed_item.ok_or("Item not found on player".to_string())?;

        // 构造响应数据
        let data = serde_json::json!({
            "player_id": player_id,
            "item": removed_item,
            "action": "remove"
        });

        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!("导演从 {} 移除了物品 {}", player.name, item_name),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 捆绑/松绑
    pub fn handle_rope_action(
        &mut self,
        player_id: &str,
        action_type: &str,
    ) -> Result<ActionResults, String> {
        // 更新指定玩家的绑定状态
        let player = self
            .players
            .get_mut(player_id)
            .ok_or("Player not found".to_string())?;
        match action_type {
            "rope" => {
                player.is_bound = true;
            }
            "unrope" => {
                player.is_bound = false;
            }
            _ => return Err("Invalid action type".to_string()),
        }

        // 构造响应数据
        let data = serde_json::json!({
            "player_id": player_id,
            "is_bound": player.is_bound
        });

        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "导演 {}  {}",
                if action_type == "rope" {
                    "捆绑"
                } else {
                    "松绑"
                },
                player.name
            ),
            true,
        );
        Ok(action_result.as_results())
    }

    /// 广播消息
    pub fn handle_broadcast(&mut self, message: &str) -> Result<ActionResults, String> {
        // 构造响应数据
        let data = serde_json::json!({
            "message": message
        });

        // 创建动作结果，广播给所有玩家和导演
        let broadcast_players: Vec<String> = self.players.keys().cloned().collect();
        let mut action_result = ActionResult::new_user_message(
            data,
            broadcast_players,
            format!("导演向全部玩家广播消息: {}", message),
            true,
        );
        action_result.broadcast_to_all = true;

        Ok(action_result.as_results())
    }

    /// 导演向特定玩家发送消息
    pub fn handle_director_message_to_player(
        &mut self,
        player_id: &str,
        message: &str,
    ) -> Result<ActionResults, String> {
        // 验证玩家是否存在
        let player_name = self
            .players
            .get_mut(player_id)
            .ok_or("Player not found".to_string())?
            .name
            .clone();

        // 构造响应数据
        let data = serde_json::json!({
            "message": format!("导演向您发送消息: {}", message)
        });

        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_user_message(
            data,
            vec![player_id.to_string()],
            format!("导演向 {} 发送消息: {}", player_name, message),
            true,
        );

        Ok(action_result.as_results())
    }
}
