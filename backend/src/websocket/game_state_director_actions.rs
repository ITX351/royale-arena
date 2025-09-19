//! GameState 导演控制实现

use super::models::*;

impl GameState {
    // ===== 导演行为处理方法 =====

    /// 设置夜晚开始时间
    pub fn handle_set_night_start_time(&mut self, timestamp: &str) -> Result<ActionResult, String> {
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
            format!("导演设置夜晚开始时间至{}", timestamp)
        );
        
        Ok(action_result)
    }

    /// 设置夜晚结束时间
    pub fn handle_set_night_end_time(&mut self, timestamp: &str) -> Result<ActionResult, String> {
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
            format!("导演设置夜晚结束时间至{}", timestamp)
        );
        
        Ok(action_result)
    }

    /// 调整地点状态
    pub fn handle_modify_place(&mut self, place_name: &str, is_destroyed: bool) -> Result<ActionResult, String> {
        // 更新指定地点的摧毁状态
        if let Some(place) = self.places.get_mut(place_name) {
            place.is_destroyed = is_destroyed;
            
            // 检查地点内的玩家是否受影响
            // 如果地点被摧毁，需要处理在该地点的玩家
            
            // 构造响应数据
            let data = serde_json::json!({
                "place": {
                    "name": place_name,
                    "is_destroyed": is_destroyed
                }
            });
            
            // 创建动作结果，只广播给导演（不需要通知玩家）
            let action_result = ActionResult::new_system_message(
                data, 
                vec![], 
                format!("导演调整地点{}状态为{}", place_name, if is_destroyed { "已摧毁" } else { "未摧毁" })
            );
            
            Ok(action_result)
        } else {
            Err("Place not found".to_string())
        }
    }

    /// 设置缩圈地点
    pub fn handle_set_destroy_places(&mut self, places: &[serde_json::Value]) -> Result<ActionResult, String> {
        // 更新下一夜晚缩圈地点集合
        self.next_night_destroyed_places = places.iter()
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
            format!("导演设置下一夜晚缩圈地点: {:?}", self.next_night_destroyed_places)
        );
        
        Ok(action_result)
    }

    /// 空投
    pub fn handle_drop(&mut self, place_name: &str, item: Item) -> Result<ActionResult, String> {
        // 在指定地点添加空投物品
        if let Some(place) = self.places.get_mut(place_name) {
            place.items.push(item);
            
            // 构造响应数据
            let data = serde_json::json!({
                "place": {
                    "name": place_name,
                    "items": place.items.clone()
                }
            });
            
            // 创建动作结果，只广播给导演（不需要通知玩家）
            let action_result = ActionResult::new_system_message(
                data, 
                vec![], 
                format!("导演在地点{}空投物品", place_name)
            );
            
            Ok(action_result)
        } else {
            Err("Place not found".to_string())
        }
    }

    /// 调整天气
    pub fn handle_weather(&mut self, weather: f64) -> Result<ActionResult, String> {
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
            format!("导演调整天气至{}", weather)
        );
        
        Ok(action_result)
    }

    /// 调整生命值
    pub fn handle_life(&mut self, player_id: &str, life_change: i64) -> Result<ActionResult, String> {
        // 更新指定玩家生命值
        if let Some(player) = self.players.get_mut(player_id) {
            player.life += life_change as i32;
            
            // 检查玩家是否死亡或复活
            if player.life <= 0 {
                player.life = 0;
                player.is_alive = false;
            } else if player.life > 0 && !player.is_alive {
                player.is_alive = true;
            }
            
            // 构造响应数据
            let data = serde_json::json!({
                "player_id": player_id,
                "life": player.life,
                "is_alive": player.is_alive
            });
            
            // 创建动作结果，广播给该玩家和所有导演
            let action_result = ActionResult::new_system_message(
                data, 
                vec![player_id.to_string()], 
                format!("导演调整玩家{}生命值{}", player_id, if life_change > 0 { format!("+{}", life_change) } else { life_change.to_string() })
            );
            
            Ok(action_result)
        } else {
            Err("Player not found".to_string())
        }
    }

    /// 调整体力值
    pub fn handle_strength(&mut self, player_id: &str, strength_change: i64) -> Result<ActionResult, String> {
        // 更新指定玩家体力值
        if let Some(player) = self.players.get_mut(player_id) {
            player.strength += strength_change as i32;
            
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
                format!("导演调整玩家{}体力值{}", player_id, if strength_change > 0 { format!("+{}", strength_change) } else { strength_change.to_string() })
            );
            
            Ok(action_result)
        } else {
            Err("Player not found".to_string())
        }
    }

    /// 移动角色
    pub fn handle_move_player(&mut self, player_id: &str, target_place: &str) -> Result<ActionResult, String> {
        // 验证目标地点是否存在且未被摧毁
        if let Some(place) = self.places.get(target_place) {
            if place.is_destroyed {
                return Err("Target place is destroyed".to_string());
            }
        } else {
            return Err("Target place not found".to_string());
        }
        
        // 获取玩家位置信息
        let player_location = {
            if let Some(player) = self.players.get(player_id) {
                player.location.clone()
            } else {
                return Err("Player not found".to_string());
            }
        };
        
        // 从当前地点移除玩家
        if !player_location.is_empty() {
            if let Some(current_place) = self.places.get_mut(&player_location) {
                current_place.players.retain(|id| id != player_id);
            }
        }
        
        // 更新玩家位置到目标地点
        if let Some(player) = self.players.get_mut(player_id) {
            player.location = target_place.to_string();
        }
        
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
            format!("导演将玩家{}移动至地点{}", player_id, target_place)
        );
        
        Ok(action_result)
    }

    /// 增减道具
    pub fn handle_give(&mut self, target_type: &str, item: Item, player_id: Option<&str>, place_name: Option<&str>) -> Result<ActionResult, String> {
        match target_type {
            "player" => {
                // 给玩家道具
                let player_id = player_id.ok_or("Missing player_id".to_string())?;
                
                if let Some(player) = self.players.get_mut(player_id) {
                    // 将物品添加到指定玩家背包
                    player.inventory.push(item);
                    
                    // 构造响应数据
                    let data = serde_json::json!({
                        "player_id": player_id,
                        "inventory": player.inventory.clone()
                    });
                    
                    // 创建动作结果，广播给该玩家和所有导演
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("导演给予玩家{}道具", player_id)
                    );
                    
                    Ok(action_result)
                } else {
                    Err("Player not found".to_string())
                }
            }
            "place" => {
                // 在地点放置道具
                let place_name = place_name.ok_or("Missing place_name".to_string())?;
                
                if let Some(place) = self.places.get_mut(place_name) {
                    // 将物品添加到指定地点物品列表
                    place.items.push(item);
                    
                    // 构造响应数据
                    let data = serde_json::json!({
                        "place": {
                            "name": place_name,
                            "items": place.items.clone()
                        }
                    });
                    
                    // 创建动作结果，只广播给导演（不需要通知玩家）
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![], 
                        format!("导演在地点{}放置道具", place_name)
                    );
                    
                    Ok(action_result)
                } else {
                    Err("Place not found".to_string())
                }
            }
            _ => Err("Invalid target type".to_string())
        }
    }

    /// 捆绑/松绑
    pub fn handle_rope_action(&mut self, player_id: &str, action_type: &str) -> Result<ActionResult, String> {
        // 更新指定玩家的绑定状态
        if let Some(player) = self.players.get_mut(player_id) {
            match action_type {
                "rope" => {
                    player.is_bound = true;
                }
                "unrope" => {
                    player.is_bound = false;
                }
                _ => return Err("Invalid action type".to_string())
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
                format!("导演{}玩家{}", 
                        if action_type == "rope" { "捆绑" } else { "松绑" }, 
                        player_id)
            );
            
            Ok(action_result)
        } else {
            Err("Player not found".to_string())
        }
    }

    /// 广播消息
    pub fn handle_broadcast(&mut self, message: &str) -> Result<ActionResult, String> {
        // 构造响应数据
        let data = serde_json::json!({
            "message": message
        });
        
        // 创建动作结果，广播给所有玩家和导演
        let broadcast_players: Vec<String> = self.players.keys().cloned().collect();
        let action_result = ActionResult::new_user_message(
            data, 
            broadcast_players, 
            format!("导演向全部玩家广播消息: {}", message)
        );
        
        Ok(action_result)
    }

    /// 导演向特定玩家发送消息
    pub fn handle_director_message_to_player(&mut self, player_id: &str, message: &str) -> Result<ActionResult, String> {
        // 验证玩家是否存在
        if !self.players.contains_key(player_id) {
            return Err("Player not found".to_string());
        }
        
        // 构造响应数据
        let data = serde_json::json!({
            "message": format!("导演向您发送消息: {}", message)
        });
        
        // 创建动作结果，广播给该玩家和所有导演
        let action_result = ActionResult::new_user_message(
            data, 
            vec![player_id.to_string()], 
            format!("导演向玩家{}发送消息: {}", player_id, message)
        );
        
        Ok(action_result)
    }
}