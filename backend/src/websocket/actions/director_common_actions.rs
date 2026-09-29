//! GameState 导演控制实现

use crate::websocket::actions::utils::{format_delta, format_delta_f64};
use crate::websocket::models::{
    ActionResult, ActionResults, AirdropItem, GameState, ItemDeletionItem, SellPriceEntry,
    ShopListing,
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
                let log_message =
                    format!("批量空投被拒绝: 物品 {} 已存在于场上", airdrop.item_name);

                let data = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "item_name": airdrop.item_name,
                    "reason": "物品已存在于场上"
                });

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
            "success_count": success_count,
            "timestamp": chrono::Utc::now().to_rfc3339()
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
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "deleted_items": deleted_items,
            "failed_items": failed_items
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
            return self.kill_player(player_id, None, None, "导演击杀");
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

    /// 设置玩家货币
    pub fn handle_set_player_coins(
        &mut self,
        player_id: &str,
        coins: f64,
    ) -> Result<ActionResults, String> {
        let (player_name, final_coins, coins_change) = {
            let player = self.players.get_mut(player_id).ok_or("Player not found")?;

            if player.coins == coins {
                let data = serde_json::json!({
                    "player_id": player_id,
                    "coins": player.coins,
                    "message": "货币未发生变化"
                });

                let log_message = format!(
                    "导演尝试设置 {} 货币为 {}，但未发生变化",
                    player.name, coins
                );

                return Ok(
                    ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
                );
            }

            let player_name = player.name.clone();
            let previous_coins = player.coins;
            player.coins = coins;
            let final_coins = player.coins;
            let coins_change = final_coins - previous_coins;

            (player_name, final_coins, coins_change)
        };

        let data = serde_json::json!({
            "player_id": player_id,
            "coins": final_coins
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "导演设置 {} 货币为 {} ({})",
                player_name,
                final_coins,
                format_delta_f64(coins_change)
            ),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 设置玩家生命上限（clamp 到 [base, cap]，当前生命高于新上限时压到新上限）
    pub fn handle_set_player_max_life(
        &mut self,
        player_id: &str,
        max_life: i32,
    ) -> Result<ActionResults, String> {
        let (base_max_life, max_life_cap) = {
            let pc = &self.rule_engine.player_config;
            (pc.max_life, pc.max_life_cap)
        };
        let cap = max_life_cap.max(base_max_life);
        let new_max_life = max_life.clamp(base_max_life, cap);

        let (player_name, final_max_life, final_life) = {
            let player = self.players.get_mut(player_id).ok_or("Player not found")?;

            if player.max_life == new_max_life {
                let data = serde_json::json!({
                    "player_id": player_id,
                    "max_life": player.max_life,
                    "message": "生命上限未发生变化"
                });
                let log_message = format!(
                    "导演尝试设置 {} 生命上限为 {}，但未发生变化（或被上限规则收敛回原值）",
                    player.name, max_life
                );
                return Ok(
                    ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
                );
            }

            let player_name = player.name.clone();
            player.max_life = new_max_life;
            if player.life > player.max_life {
                player.life = player.max_life;
            }
            (player_name, player.max_life, player.life)
        };

        let data = serde_json::json!({
            "player_id": player_id,
            "max_life": final_max_life,
            "life": final_life
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "导演设置 {} 生命上限为 {}（当前生命 {}）",
                player_name, final_max_life, final_life
            ),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 设置玩家体力上限（clamp 到 [base, cap]，当前体力高于新上限时压到新上限）
    pub fn handle_set_player_max_strength(
        &mut self,
        player_id: &str,
        max_strength: i32,
    ) -> Result<ActionResults, String> {
        let (base_max_strength, max_strength_cap) = {
            let pc = &self.rule_engine.player_config;
            (pc.max_strength, pc.max_strength_cap)
        };
        let cap = max_strength_cap.max(base_max_strength);
        let new_max_strength = max_strength.clamp(base_max_strength, cap);

        let (player_name, final_max_strength, final_strength) = {
            let player = self.players.get_mut(player_id).ok_or("Player not found")?;

            if player.max_strength == new_max_strength {
                let data = serde_json::json!({
                    "player_id": player_id,
                    "max_strength": player.max_strength,
                    "message": "体力上限未发生变化"
                });
                let log_message = format!(
                    "导演尝试设置 {} 体力上限为 {}，但未发生变化（或被上限规则收敛回原值）",
                    player.name, max_strength
                );
                return Ok(
                    ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
                );
            }

            let player_name = player.name.clone();
            player.max_strength = new_max_strength;
            if player.strength > player.max_strength {
                player.strength = player.max_strength;
            }
            (player_name, player.max_strength, player.strength)
        };

        let data = serde_json::json!({
            "player_id": player_id,
            "max_strength": final_max_strength,
            "strength": final_strength
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "导演设置 {} 体力上限为 {}（当前体力 {}）",
                player_name, final_max_strength, final_strength
            ),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 设置玩家背包上限（clamp 到 [base, cap]，降低上限不丢弃已有物品）
    pub fn handle_set_player_max_backpack(
        &mut self,
        player_id: &str,
        max_backpack_items: i32,
    ) -> Result<ActionResults, String> {
        let (base_backpack, backpack_cap) = {
            let pc = &self.rule_engine.player_config;
            (pc.max_backpack_items, pc.max_backpack_items_cap)
        };
        let base = base_backpack as i32;
        let cap = (backpack_cap as i32).max(base);
        let new_max = max_backpack_items.clamp(base, cap) as usize;

        let (player_name, final_max) = {
            let player = self.players.get_mut(player_id).ok_or("Player not found")?;

            if player.max_backpack_items == new_max {
                let data = serde_json::json!({
                    "player_id": player_id,
                    "max_backpack_items": player.max_backpack_items,
                    "message": "背包上限未发生变化"
                });
                let log_message = format!(
                    "导演尝试设置 {} 背包上限为 {}，但未发生变化（或被上限规则收敛回原值）",
                    player.name, max_backpack_items
                );
                return Ok(
                    ActionResult::new_info_message(data, vec![], log_message, true).as_results(),
                );
            }

            let player_name = player.name.clone();
            player.max_backpack_items = new_max;
            (player_name, new_max)
        };

        // 降低上限不丢弃已有物品：inventory 不裁剪，仅拦截后续拾取

        let data = serde_json::json!({
            "player_id": player_id,
            "max_backpack_items": final_max
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!("导演设置 {} 背包上限为 {}", player_name, final_max),
            true,
        );

        Ok(action_result.as_results())
    }

    /// 移动玩家到指定地点
    pub fn handle_move_player(
        &mut self,
        player_id: &str,
        target_place: &str,
    ) -> Result<ActionResults, String> {
        let player = self.players.get(player_id).ok_or("Player not found")?;

        let current_location = player.location.clone();
        let player_name = player.name.clone();

        if current_location == target_place {
            let data = serde_json::json!({
                "player_id": player_id,
                "location": current_location,
                "message": "位置未发生变化"
            });
            return Ok(ActionResult::new_info_message(
                data,
                vec![],
                format!("导演尝试移动 {}，但位置未发生变化", player_name),
                true,
            )
            .as_results());
        }

        // 验证目标地点是否存在
        if !self.places.contains_key(target_place) {
            return Err(format!("目标地点 {} 不存在", target_place));
        }

        // 验证目标地点是否被摧毁
        if self.places[target_place].is_destroyed {
            let data = serde_json::json!({
                "player_id": player_id,
                "target_place": target_place,
                "reason": "地点已被摧毁",
            });
            return Ok(ActionResult::new_info_message(
                data,
                vec![],
                format!("无法移动 {} 到 {}：地点已被摧毁", player_name, target_place),
                true,
            )
            .as_results());
        }

        // 从旧地点移除玩家
        if !current_location.is_empty() {
            if let Some(place) = self.places.get_mut(&current_location) {
                place.players.retain(|id| id != player_id);
            }
        }

        // 更新玩家位置
        let player = self.players.get_mut(player_id).unwrap();
        player.location = target_place.to_string();

        // 将玩家添加到新地点
        let place = self.places.get_mut(target_place).unwrap();
        place.players.push(player_id.to_string());

        let data = serde_json::json!({
            "player_id": player_id,
            "location": target_place
        });

        let action_result = ActionResult::new_system_message(
            data,
            vec![player_id.to_string()],
            format!(
                "导演移动 {} 从 {} 到 {}",
                player_name, current_location, target_place
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
            let log_message = format!("无法添加物品 {}: 该物品已存在于场上", item_name);

            let data = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "item_name": item_name,
                "reason": "该物品已存在于场上"
            });

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

    /// 商店上架物品
    pub fn handle_shop_list_item(
        &mut self,
        item_name: String,
        price: i32,
        quantity: i32,
    ) -> Result<ActionResults, String> {
        // 验证价格合法性
        if price < 1 {
            let data = serde_json::json!({});
            return Ok(ActionResult::new_info_message(
                data,
                vec![],
                format!("上架价格必须 >= 1，当前值为 {}", price),
                true,
            )
            .as_results());
        }

        // 验证物品名称是否存在于规则配置中
        let preview = self
            .rule_engine
            .create_item_from_name(&item_name)
            .map_err(|err| format!("物品 {} 不存在于规则配置中: {}", item_name, err))?;
        if preview.is_weapon_or_armor() {
            let data = serde_json::json!({});
            return Ok(ActionResult::new_info_message(
                data,
                vec![],
                "武器和防具请按稀有度类目上架".to_string(),
                true,
            )
            .as_results());
        }

        let qty = quantity.max(1);

        let listing = ShopListing {
            id: uuid::Uuid::new_v4().to_string(),
            item_name,
            price,
            quantity: qty,
            item_kind: None,
            rarity: None,
        };

        self.shop.push(listing.clone());

        let data = serde_json::json!({
            "shop_listing": listing,
        });

        let broadcast_players: Vec<String> = self.players.keys().cloned().collect();
        let mut action_result = ActionResult::new_system_message(
            data,
            broadcast_players,
            format!(
                "导演上架物品: {} x{} (单价: {})",
                listing.item_name, listing.quantity, listing.price
            ),
            true,
        );
        action_result.broadcast_to_all = true;

        Ok(action_result.as_results())
    }

    /// 商店下架物品
    pub fn handle_shop_delist_item(&mut self, listing_id: &str) -> Result<ActionResults, String> {
        let pos = self
            .shop
            .iter()
            .position(|l| l.id == listing_id)
            .ok_or("商品未找到".to_string())?;

        let removed = self.shop.remove(pos);

        let data = serde_json::json!({
            "shop_listing_id": listing_id,
        });

        let broadcast_players: Vec<String> = self.players.keys().cloned().collect();
        let mut action_result = ActionResult::new_system_message(
            data,
            broadcast_players,
            format!("导演下架物品: {}", removed.item_name),
            true,
        );
        action_result.broadcast_to_all = true;

        Ok(action_result.as_results())
    }

    /// 导演更新队友行为配置
    pub fn handle_set_teammate_behavior(&mut self, mode: i32) -> Result<ActionResults, String> {
        // 1. 写回 rule_engine（让本轮后续 action 立刻生效）
        self.rule_engine.teammate_behavior.mode = mode;

        // 2. 同步到 rules_config JSON（让前端 state_update 看到新值、让存档保留）
        if let Some(obj) = self.rules_config.as_object_mut() {
            obj.insert("teammate_behavior".to_string(), serde_json::json!(mode));
        }

        // 3. 全员广播 SystemNotice
        let all_player_ids: Vec<String> = self.players.keys().cloned().collect();
        Ok(ActionResult::new_system_message(
            serde_json::json!({ "teammate_behavior": mode }),
            all_player_ids,
            format!("导演已更新队友行为规则（位掩码={}）", mode),
            true,
        )
        .as_results())
    }

    /// 售出价格校验与显示辅助
    pub fn rarity_display_name(rarity: &str) -> Option<&'static str> {
        match rarity {
            "common" => Some("绿"),
            "rare" => Some("蓝"),
            "epic" => Some("紫"),
            "legendary" => Some("橙"),
            _ => None,
        }
    }

    /// 商店稀有度类目：类别中文名
    pub fn item_kind_display_name(kind: &str) -> Option<&'static str> {
        match kind {
            "weapon" => Some("武器"),
            "armor" => Some("防具"),
            _ => None,
        }
    }

    /// 道具库中该类别该稀有度的全部候选显示名（多条目合并）
    pub fn rarity_display_names(&self, item_kind: &str, rarity: &str) -> Vec<String> {
        match item_kind {
            "weapon" => self
                .rule_engine
                .items_config
                .items
                .weapons
                .iter()
                .filter(|w| w.rarity.as_deref() == Some(rarity))
                .flat_map(|w| w.display_names.clone())
                .collect(),
            "armor" => self
                .rule_engine
                .items_config
                .items
                .armors
                .iter()
                .filter(|a| a.rarity.as_deref() == Some(rarity))
                .flat_map(|a| a.display_names.clone())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// 商店上架稀有度类目（武器/防具 × 稀有度，购买时随机抽取）
    pub fn handle_shop_list_rarity(
        &mut self,
        item_kind: String,
        rarity: String,
        price: i32,
        quantity: i32,
    ) -> Result<ActionResults, String> {
        let info = |message: String| -> ActionResults {
            ActionResult::new_info_message(serde_json::json!({}), vec![], message, true).as_results()
        };
        let Some(kind_cn) = Self::item_kind_display_name(&item_kind) else {
            return Ok(info(format!(
                "类目必须为 weapon 或 armor，当前值为 {}",
                item_kind
            )));
        };
        let Some(rarity_cn) = Self::rarity_display_name(&rarity) else {
            return Ok(info(format!(
                "稀有度必须为 common/rare/epic/legendary，当前值为 {}",
                rarity
            )));
        };

        if price < 1 {
            return Ok(info(format!("上架价格必须 >= 1，当前值为 {}", price)));
        }

        let pool = self.rarity_display_names(&item_kind, &rarity);
        if pool.is_empty() {
            return Ok(info(format!("道具库中没有{}类{}", rarity_cn, kind_cn)));
        }

        let qty = quantity.max(1);
        if qty > pool.len() as i32 {
            return Ok(info(format!(
                "库存不能超过道具库中{}类{}的名称总数（{}）",
                rarity_cn,
                kind_cn,
                pool.len()
            )));
        }

        if self
            .shop
            .iter()
            .any(|l| l.item_kind.as_deref() == Some(item_kind.as_str())
                && l.rarity.as_deref() == Some(rarity.as_str()))
        {
            return Ok(info("该类目已上架".to_string()));
        }

        let listing = ShopListing {
            id: uuid::Uuid::new_v4().to_string(),
            item_name: format!("{}类{}（随机）", rarity_cn, kind_cn),
            price,
            quantity: qty,
            item_kind: Some(item_kind),
            rarity: Some(rarity),
        };
        self.shop.push(listing.clone());

        let data = serde_json::json!({ "shop_listing": listing });
        let broadcast_players: Vec<String> = self.players.keys().cloned().collect();
        let mut action_result = ActionResult::new_system_message(
            data,
            broadcast_players,
            format!(
                "导演上架了{}类{}（随机），价格 {} 货币，库存 {}",
                rarity_cn, kind_cn, price, qty
            ),
            true,
        );
        action_result.broadcast_to_all = true;

        Ok(action_result.as_results())
    }

    /// 格式化价格显示（整数省略小数点）
    pub fn format_price(price: f64) -> String {
        if (price - price.round()).abs() < 1e-9 {
            format!("{}", price.round() as i64)
        } else {
            format!("{}", price)
        }
    }

    /// 导演设置售出价格（新增或改价）
    pub fn handle_sell_set_price(
        &mut self,
        rarity: String,
        price: f64,
    ) -> Result<ActionResults, String> {
        let Some(display) = Self::rarity_display_name(&rarity) else {
            return Ok(ActionResult::new_info_message(
                serde_json::json!({}),
                vec![],
                format!(
                    "稀有度必须为 common/rare/epic/legendary，当前值为 {}",
                    rarity
                ),
                true,
            )
            .as_results());
        };
        if price <= 0.0 || (price * 2.0 - (price * 2.0).round()).abs() >= 1e-9 {
            return Ok(ActionResult::new_info_message(
                serde_json::json!({}),
                vec![],
                format!("售出价格必须为 0.5 的倍数且大于 0，当前值为 {}", price),
                true,
            )
            .as_results());
        }

        let broadcast_rarity = rarity.clone();
        if let Some(entry) = self.sell_prices.iter_mut().find(|e| e.rarity == rarity) {
            entry.price = price;
        } else {
            self.sell_prices.push(SellPriceEntry {
                id: uuid::Uuid::new_v4().to_string(),
                rarity,
                price,
            });
        }

        let broadcast_players: Vec<String> = self.players.keys().cloned().collect();
        let mut action_result = ActionResult::new_system_message(
            serde_json::json!({ "rarity": broadcast_rarity, "price": price }),
            broadcast_players,
            format!(
                "导演更新了售出价格：{}类道具 {} 货币",
                display,
                Self::format_price(price)
            ),
            true,
        );
        action_result.broadcast_to_all = true;
        Ok(action_result.as_results())
    }

    /// 导演删除某稀有度的售出配置
    pub fn handle_sell_remove_price(&mut self, rarity: &str) -> Result<ActionResults, String> {
        let Some(display) = Self::rarity_display_name(rarity) else {
            return Ok(ActionResult::new_info_message(
                serde_json::json!({}),
                vec![],
                format!(
                    "稀有度必须为 common/rare/epic/legendary，当前值为 {}",
                    rarity
                ),
                true,
            )
            .as_results());
        };
        let Some(pos) = self.sell_prices.iter().position(|e| e.rarity == rarity) else {
            return Ok(ActionResult::new_info_message(
                serde_json::json!({}),
                vec![],
                format!("{}类道具尚未配置售出价格", display),
                true,
            )
            .as_results());
        };
        let rarity = rarity.to_string();
        self.sell_prices.remove(pos);

        let broadcast_players: Vec<String> = self.players.keys().cloned().collect();
        let mut action_result = ActionResult::new_system_message(
            serde_json::json!({ "rarity": rarity }),
            broadcast_players,
            format!("导演关闭了{}类道具的售出", display),
            true,
        );
        action_result.broadcast_to_all = true;
        Ok(action_result.as_results())
    }
}
