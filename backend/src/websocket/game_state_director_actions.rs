//! GameState 导演控制实现

use super::models::*;

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

    /// 调整地点状态
    pub fn handle_modify_place(
        &mut self,
        place_name: &str,
        is_destroyed: bool,
    ) -> Result<ActionResults, String> {
        // 更新指定地点的摧毁状态，同时记录需要处理的玩家
        let players_to_kill = {
            let place = self.places.get_mut(place_name).ok_or("Place not found")?;
            place.is_destroyed = is_destroyed;
            if is_destroyed {
                place.players.clone()
            } else {
                Vec::new()
            }
        };

        // 构造响应数据
        let data = serde_json::json!({
            "place": {
                "name": place_name,
                "is_destroyed": is_destroyed
            }
        });

        let mut results = vec![ActionResult::new_system_message(
            data,
            vec![],
            format!(
                "导演调整地点 {} 状态为 {}",
                place_name,
                if is_destroyed {
                    "已摧毁"
                } else {
                    "未摧毁"
                }
            ),
            true,
        )];

        if is_destroyed {
            for player_id in players_to_kill {
                let mut death_outcome = self.kill_player(&player_id, None, "缩圈")?;
                results.append(&mut death_outcome.results);
            }
        }

        Ok(ActionResults { results })
    }

    /// 设置缩圈地点
    pub fn handle_set_destroy_places(
        &mut self,
        places: &[serde_json::Value],
    ) -> Result<ActionResults, String> {
        // 更新下一夜晚缩圈地点集合
        self.next_night_destroyed_places = places
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();

        // 构造响应数据
        let data = serde_json::json!({
            "next_night_destroyed_places": self.next_night_destroyed_places.clone()
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

        if current_life == 0 && !is_alive && life > 0 {
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
                if life_change > 0 {
                    format!("+{}", life_change)
                } else {
                    life_change.to_string()
                }
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
        // 更新指定玩家体力值
        let player = self.players.get_mut(player_id).ok_or("Player not found")?;

        // 检查体力值是否发生变化
        if player.strength == strength {
            // 如果没有变化，返回Info消息
            let data = serde_json::json!({
                "player_id": player_id,
                "strength": player.strength,
                "message": "体力值未发生变化"
            });

            let log_message = format!(
                "导演尝试设置 {} 体力值为 {}，但未发生变化",
                player.name, strength
            );

            // 创建Info类型的动作结果，只广播给导演
            return Ok(
                ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
            );
        }

        let strength_change = strength - player.strength;
        player.strength = strength;

        // 确保体力值在合理范围内
        if player.strength < 0 {
            player.strength = 0;
        }

        // 构造响应数据
        let data = serde_json::json!({
            "player_id": player_id,
            "strength": player.strength
        });

        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "导演设置 {} 体力值为 {} ({})",
                player.name,
                strength,
                if strength_change > 0 {
                    format!("+{}", strength_change)
                } else {
                    strength_change.to_string()
                }
            ),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 移动角色
    pub fn handle_move_player(
        &mut self,
        player_id: &str,
        target_place: &str,
    ) -> Result<ActionResults, String> {
        // 验证目标地点是否存在且未被摧毁
        if let Some(place) = self.places.get(target_place) {
            if place.is_destroyed {
                return Err("Target place is destroyed".to_string());
            }
        } else {
            return Err("Target place not found".to_string());
        }

        // 获取玩家位置信息
        let player = self
            .players
            .get_mut(player_id)
            .ok_or("Player not found".to_string())?;

        // 从当前地点移除玩家
        if !player.location.is_empty() {
            if let Some(current_place) = self.places.get_mut(&player.location) {
                current_place.players.retain(|id| id != player_id);
            }
        }

        // 更新玩家位置到目标地点
        player.location = target_place.to_string();

        // 将玩家添加到目标地点的玩家列表中
        if let Some(target_place_obj) = self.places.get_mut(target_place) {
            target_place_obj.players.push(player_id.to_string());
        }

        // 构造响应数据
        let data = serde_json::json!({
            "player_id": player_id,
            "location": target_place
        });

        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!("导演将 {} 移动至地点 {}", player.name, target_place),
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

        // 查找并移除物品
        let item_pos = player
            .inventory
            .iter()
            .position(|i| i.name == item_name)
            .ok_or("Item not found in player's inventory".to_string())?;
        let removed_item = player.inventory.remove(item_pos);

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
