//! GameState 玩家行动实现

use super::models::*;

impl GameState {
    /// 处理玩家出生行动
    pub fn handle_born_action(&mut self, player_id: &str, place_name: &str) -> Result<ActionResult, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查玩家是否已经执行过出生
        if !player.location.is_empty() {
            return Err("Player has already been born".to_string());
        }
        
        // 验证指定地点是否存在且未被摧毁
        if let Some(place) = self.places.get(place_name) {
            if place.is_destroyed {
                return Err("Place is destroyed".to_string());
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
                format!("玩家{}在地点{}出生", player.name, place_name)
            );
            
            Ok(action_result)
        } else {
            Err("Place not found".to_string())
        }
    }

    /// 处理玩家移动行动
    pub fn handle_move_action(&mut self, player_id: &str, target_place: &str) -> Result<ActionResult, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，有足够体力
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 验证目标地点是否存在且未被摧毁
        if let Some(place) = self.places.get(target_place) {
            if place.is_destroyed {
                return Err("Target place is destroyed".to_string());
            }
            
            // 消耗体力值（根据规则配置）
            // 这里我们简化处理，假设每次移动消耗5点体力
            let move_cost = 5;
            if player.strength < move_cost {
                return Err("Not enough strength".to_string());
            }
            player.strength -= move_cost;
            
            // 从当前地点移除玩家
            if let Some(current_place) = self.places.get_mut(&player.location) {
                current_place.players.retain(|id| id != &player.id);
            }
            
            // 更新玩家位置到目标地点
            player.location = target_place.to_string();
            
            // 将玩家添加到目标地点的玩家列表中
            if let Some(target_place_obj) = self.places.get_mut(target_place) {
                target_place_obj.players.push(player.id.clone());
            }
            
            // 向该玩家发送位置更新结果
            let data = serde_json::json!({
                "location": target_place,
                "strength": player.strength
            });
            
            // 创建动作结果，只广播给发起者本人
            let action_result = ActionResult::new_system_message(
                data, 
                vec![player_id.to_string()], 
                format!("玩家{}移动到地点{}", player.name, target_place)
            );
            
            Ok(action_result)
        } else {
            Err("Target place not found".to_string())
        }
    }

    /// 处理搜索行动
    pub fn handle_search_action(&mut self, player_id: &str) -> Result<ActionResult, String> {
        // 获取玩家状态信息（避免借用冲突）
        let (player_location, player_alive, player_strength, player_last_search_time) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            (
                player.location.clone(),
                player.is_alive,
                player.strength,
                player.last_search_time,
            )
        };
        
        // 检查前置条件：玩家处于存活状态，有足够体力，未处于搜索冷却期
        if !player_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 检查体力
        let search_cost = 5; // 假设搜索消耗5点体力
        if player_strength < search_cost {
            return Err("Not enough strength".to_string());
        }
        
        // 检查搜索冷却期（简化处理，假设冷却期为30秒）
        let search_cooldown = 30;
        if let Some(last_search_time) = player_last_search_time {
            let elapsed = chrono::Utc::now().signed_duration_since(last_search_time);
            if elapsed.num_seconds() < search_cooldown {
                return Err("Search is in cooldown period".to_string());
            }
        }
        
        // 更新玩家状态
        {
            let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            player.strength -= search_cost;
            player.last_search_time = Some(chrono::Utc::now());
        }
        
        // 随机确定搜索结果（玩家/物品/空）
        // 这里我们简化处理，随机生成结果
        use rand::Rng;
        let mut rng = rand::rng(); // 使用正确的rng方法
        let result_type: u32 = rng.random_range(0..3);
        
        match result_type {
            0 => {
                // 搜索到玩家
                // 查找当前地点的其他玩家（先获取地点信息，避免借用冲突）
                let other_player_ids = {
                    // 先检查地点是否存在
                    if self.places.contains_key(&player_location) {
                        // 获取地点玩家列表的副本
                        let place = &self.places[&player_location];
                        let mut place_players = Vec::new();
                        for id in &place.players {
                            place_players.push(id.clone());
                        }
                        
                        // 过滤掉当前玩家
                        let mut filtered_ids = Vec::new();
                        for id in place_players {
                            if id != player_id {
                                filtered_ids.push(id.clone());
                            }
                        }
                        filtered_ids
                    } else {
                        Vec::new()
                    }
                };
                
                if !other_player_ids.is_empty() {
                    // 随机选择一个玩家
                    let target_player_id = &other_player_ids[rng.random_range(0..other_player_ids.len())];
                    
                    // 获取目标玩家信息
                    let (target_player_id_clone, target_player_name) = {
                        if let Some(target_player) = self.players.get(target_player_id) {
                            (target_player.id.clone(), target_player.name.clone())
                        } else {
                            return Err("Target player not found".to_string());
                        }
                    };
                    
                    // 根据天气条件确定结果可见性
                    let is_visible = rng.random_bool(self.weather);
                    
                    // 更新玩家的上次搜索结果
                    {
                        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                        player.last_search_result = Some(SearchResult {
                            target_type: SearchResultType::Player,
                            target_id: target_player_id_clone.clone(),
                            target_name: target_player_name.clone(),
                            is_visible,
                        });
                    }
                    
                    // 获取更新后的玩家状态
                    let (player_strength, player_last_search_time) = {
                        let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
                        (player.strength, player.last_search_time)
                    };
                    
                    // 向该玩家发送搜索结果
                    let data = serde_json::json!({
                        "last_search_result": {
                            "target_type": "player",
                            "target_id": target_player_id_clone,
                            "target_name": target_player_name,
                            "is_visible": is_visible
                        },
                        "strength": player_strength,
                        "last_search_time": player_last_search_time
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家{}搜索并发现了玩家{}", 
                            self.players.get(player_id).unwrap().name, target_player_name)
                    );
                    
                    return Ok(action_result);
                }
                
                // 如果没有其他玩家，返回空结果
                {
                    let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                    player.last_search_result = None;
                }
                
                // 获取更新后的玩家状态
                let (player_strength, player_last_search_time) = {
                    let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
                    (player.strength, player.last_search_time)
                };
                
                let data = serde_json::json!({
                    "last_search_result": null,
                    "strength": player_strength,
                    "last_search_time": player_last_search_time
                });
                
                // 创建动作结果，只广播给发起者本人
                let action_result = ActionResult::new_system_message(
                    data, 
                    vec![player_id.to_string()], 
                    format!("玩家{}搜索但没有发现任何东西", 
                        self.players.get(player_id).unwrap().name)
                );
                Ok(action_result)
            }
            1 => {
                // 搜索到物品
                // 先获取地点信息，避免借用冲突
                let place_info = {
                    if let Some(place) = self.places.get(&player_location) {
                        if !place.items.is_empty() {
                            // 随机选择一个物品
                            let item_index = rng.random_range(0..place.items.len());
                            let item = &place.items[item_index];
                            Some((item.id.clone(), item.name.clone()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                
                if let Some((item_id, item_name)) = place_info {
                    // 根据天气条件确定结果可见性
                    let is_visible = rng.random_bool(self.weather);
                    
                    // 更新玩家的上次搜索结果
                    {
                        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                        player.last_search_result = Some(SearchResult {
                            target_type: SearchResultType::Item,
                            target_id: item_id.clone(),
                            target_name: item_name.clone(),
                            is_visible,
                        });
                    }
                    
                    // 获取更新后的玩家状态
                    let (player_strength, player_last_search_time) = {
                        let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
                        (player.strength, player.last_search_time)
                    };
                    
                    // 向该玩家发送搜索结果
                    let data = serde_json::json!({
                        "last_search_result": {
                            "target_type": "item",
                            "target_id": item_id,
                            "target_name": item_name,
                            "is_visible": is_visible
                        },
                        "strength": player_strength,
                        "last_search_time": player_last_search_time
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家{}搜索并发现了物品{}", 
                            self.players.get(player_id).unwrap().name, item_name)
                    );
                    Ok(action_result)
                } else {
                    // 地点没有物品，返回空结果
                    {
                        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                        player.last_search_result = None;
                    }
                    
                    // 获取更新后的玩家状态
                    let (player_strength, player_last_search_time) = {
                        let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
                        (player.strength, player.last_search_time)
                    };
                    
                    let data = serde_json::json!({
                        "last_search_result": null,
                        "strength": player_strength,
                        "last_search_time": player_last_search_time
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家{}搜索但没有发现任何东西", 
                            self.players.get(player_id).unwrap().name));
                    Ok(action_result)
                }
            }
            _ => {
                // 搜索结果为空
                {
                    let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                    player.last_search_result = None;
                }
                
                // 获取更新后的玩家状态
                let (player_strength, player_last_search_time) = {
                    let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
                    (player.strength, player.last_search_time)
                };
                
                let data = serde_json::json!({
                    "last_search_result": null,
                    "strength": player_strength,
                    "last_search_time": player_last_search_time
                });
                
                // 创建动作结果，只广播给发起者本人
                let action_result = ActionResult::new_system_message(
                    data, 
                    vec![player_id.to_string()], 
                    format!("玩家{}搜索但没有发现任何东西", 
                            self.players.get(player_id).unwrap().name));
                Ok(action_result)
            }
        }
    }

    /// 处理捡拾行动
    pub fn handle_pick_action(&mut self, player_id: &str) -> Result<ActionResult, String> {
        // 检查玩家是否存在且处于存活状态，上一次搜索结果为物品
        {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            
            if !player.is_alive {
                return Err("Player is not alive".to_string());
            }
            
            let last_search_result_valid = if let Some(ref search_result) = player.last_search_result {
                search_result.target_type == SearchResultType::Item
            } else {
                false
            };
            
            if !last_search_result_valid {
                return Err("Last search result is not an item".to_string());
            }
        }
        
        // 获取搜索结果信息和玩家位置
        let (player_last_search_result, player_location) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
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
                
                // 将物品添加到玩家背包
                {
                    let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                    player.inventory.push(item);
                }
                
                // 获取更新后的玩家背包
                let player_inventory = {
                    let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
                    player.inventory.clone()
                };
                
                // 向该玩家发送背包更新
                let data = serde_json::json!({
                    "inventory": player_inventory
                });
                
                // 创建动作结果，只广播给发起者本人
                let action_result = ActionResult::new_system_message(
                    data, 
                    vec![player_id.to_string()], 
                    format!("玩家{}捡起了一个物品", 
                        self.players.get(player_id).unwrap().name)
                );
                Ok(action_result)
            } else {
                // 物品不存在，向该玩家发送捡拾失败消息
                let data = serde_json::json!({
                    "message": "Item no longer exists"
                });
                
                // 创建动作结果，只广播给发起者本人
                let action_result = ActionResult::new_system_message(
                    data, 
                    vec![player_id.to_string()], 
                    format!("玩家{}试图捡起一个物品但该物品已不存在", 
                        self.players.get(player_id).unwrap().name)
                );
                Ok(action_result)
            }
        } else {
            Err("Player location not found".to_string())
        }
    }

    /// 处理攻击行动
    pub fn handle_attack_action(&mut self, player_id: &str) -> Result<ActionResult, String> {
        // 检查前置条件：玩家处于存活状态，上一次搜索结果为玩家，玩家装备了武器
        {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            
            if !player.is_alive {
                return Err("Player is not alive".to_string());
            }
            
            if player.equipped_item.is_none() {
                return Err("Player has no equipped item".to_string());
            }
            
            if player.last_search_result.is_none() {
                return Err("No previous search result".to_string());
            }
            
            let last_search_result_valid = if let Some(ref search_result) = player.last_search_result {
                search_result.target_type == SearchResultType::Player
            } else {
                false
            };
            
            if !last_search_result_valid {
                return Err("Last search result is not a player".to_string());
            }
        }
        
        // 获取搜索结果信息和玩家位置
        let (player_last_search_result, player_location) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            (player.last_search_result.clone(), player.location.clone())
        };
        
        // 获取搜索结果信息
        let target_player_id = if let Some(ref search_result) = player_last_search_result {
            search_result.target_id.clone()
        } else {
            return Err("No previous search result".to_string());
        };
        
        // 验证目标玩家是否存在且在同一地点
        let (target_player_location, target_player_alive) = {
            if let Some(target_player) = self.players.get(&target_player_id) {
                (target_player.location.clone(), target_player.is_alive)
            } else {
                return Err("Target player not found".to_string());
            }
        };
        
        // 验证目标玩家是否在同一地点
        if target_player_location != player_location {
            // 目标玩家已离开
            let data = serde_json::json!({
                "message": "Target player has left the location"
            });
            
            // 创建动作结果，只广播给发起者本人
            let action_result = ActionResult::new_system_message(
                data, 
                vec![player_id.to_string()], 
                format!("玩家{}试图攻击但目标玩家已离开该地点", 
                    self.players.get(player_id).unwrap().name)
            );
            return Ok(action_result);
        }
        
        // 检查目标玩家是否已死亡
        if !target_player_alive {
            // 目标玩家已死亡
            let data = serde_json::json!({
                "message": "Target player is already dead"
            });
            
            // 创建动作结果，只广播给发起者本人
            let action_result = ActionResult::new_system_message(
                data, 
                vec![player_id.to_string()], 
                format!("玩家{}试图攻击但目标玩家已经死亡", 
                    self.players.get(player_id).unwrap().name)
            );
            return Ok(action_result);
        }
        
        // 根据武器属性计算伤害（简化处理）
        let damage = 20; // 假设固定伤害值
        
        // 减少目标玩家生命值
        {
            let target_player = self.players.get_mut(&target_player_id).ok_or("Target player not found".to_string())?;
            target_player.life -= damage;
            
            // 检查目标玩家是否死亡
            if target_player.life <= 0 {
                target_player.life = 0;
                target_player.is_alive = false;
            }
        } // 释放对目标玩家的可变借用
        
        // 获取目标玩家的当前状态
        let (target_player_life, target_player_is_alive) = {
            let target_player = self.players.get(&target_player_id).ok_or("Target player not found".to_string())?;
            (target_player.life, target_player.is_alive)
        };
        
        // 向攻击者发送攻击结果（仅包括主目标）
        let data = serde_json::json!({
            "message": format!("Attacked player {} for {} damage", target_player_id, damage),
            "target_player_life": target_player_life,
            "target_player_is_alive": target_player_is_alive
        });
        
        // 向被攻击者发送被攻击通知（不包括攻击者身份）
        let _target_data = serde_json::json!({
            "message": format!("You were attacked for {} damage", damage)
        });
        
        // 消耗体力值（根据规则配置，假设攻击消耗10点体力）
        let attack_cost = 10;
        {
            let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            if player.strength >= attack_cost {
                player.strength -= attack_cost;
            } else {
                player.strength = 0;
            }
        } // 释放对攻击者的可变借用
        
        // 创建动作结果，广播给攻击者和被攻击者
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string(), target_player_id.clone()], 
            format!("玩家{}攻击玩家{}造成{}点伤害", 
                self.players.get(player_id).unwrap().name, target_player_id, damage)
        );
        
        Ok(action_result)
    }

    /// 处理装备行动
    pub fn handle_equip_action(&mut self, player_id: &str, item_id: &str) -> Result<ActionResult, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，背包中有指定物品
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 验证玩家背包中是否存在指定物品
        if let Some(_item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            // 更新玩家当前手持物品
            player.hand_item = Some(item_id.to_string());
            
            // 向该玩家发送手持物品状态更新
            let data = serde_json::json!({
                "hand_item": item_id
            });
            
            // 创建动作结果，只广播给发起者本人
            let action_result = ActionResult::new_system_message(
                data, 
                vec![player_id.to_string()], 
                format!("玩家{}装备了物品{}", player.name, item_id)
            );
            Ok(action_result)
        } else {
            Err("Item not found in player's inventory".to_string())
        }
    }

    /// 处理使用道具行动
    pub fn handle_use_action(&mut self, player_id: &str, item_id: &str) -> Result<ActionResult, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，手持道具
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        if player.hand_item.is_none() {
            return Err("Player has no item in hand".to_string());
        }
        
        // 验证手持的是否是指定物品
        if player.hand_item.as_ref() != Some(&item_id.to_string()) {
            return Err("Specified item is not in player's hand".to_string());
        }
        
        // 查找物品信息
        if let Some(item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            let item = &player.inventory[item_index];
            
            // 根据道具类型执行相应效果
            match item.item_type {
                ItemType::Consumable => {
                    // 如果是消耗品：恢复生命值、传送等，并从玩家背包中移除消耗品
                    // 这里我们简化处理，假设所有消耗品都恢复20点生命值
                    player.life += 20;
                    if player.life > 100 {
                        player.life = 100; // 假设最大生命值为100
                    }
                    
                    // 从玩家背包中移除消耗品
                    player.inventory.remove(item_index);
                    
                    // 清空手持物品
                    player.hand_item = None;
                    
                    // 更新玩家状态
                    let data = serde_json::json!({
                        "life": player.life,
                        "inventory": player.inventory,
                        "hand_item": null
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家{}使用了消耗品{}", player.name, item_id)
                    );
                    Ok(action_result)
                }
                ItemType::Weapon => {
                    // 如果是装备类：装备到对应位置，替换原有装备
                    // 这里我们简化处理，只是更新装备状态
                    player.equipped_item = Some(item_id.to_string());
                    
                    // 更新玩家状态
                    let data = serde_json::json!({
                        "equipped_item": item_id
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家{}装备了武器{}", player.name, item_id)
                    );
                    Ok(action_result)
                }
                ItemType::Equipment => {
                    // 其他装备类型处理
                    player.equipped_item = Some(item_id.to_string());
                    
                    // 更新玩家状态
                    let data = serde_json::json!({
                        "equipped_item": item_id
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家{}装备了物品{}", player.name, item_id)
                    );
                    Ok(action_result)
                }
            }
        } else {
            Err("Item not found in player's inventory".to_string())
        }
    }

    /// 处理丢弃道具行动
    pub fn handle_throw_action(&mut self, player_id: &str, item_id: &str) -> Result<ActionResult, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，背包中有指定物品
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 验证玩家背包中是否存在指定物品
        if let Some(item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            // 从玩家背包中移除物品
            let item = player.inventory.remove(item_index);
            
            // 将物品添加到当前地点的物品列表
            if let Some(place) = self.places.get_mut(&player.location) {
                place.items.push(item);
            }
            
            // 如果丢弃的是手持物品，清空手持物品状态
            if player.hand_item.as_ref() == Some(&item_id.to_string()) {
                player.hand_item = None;
            }
            
            // 向该玩家发送背包更新
            let data = serde_json::json!({
                "inventory": player.inventory,
                "hand_item": player.hand_item
            });
            
            // 创建动作结果，只广播给发起者本人
            let action_result = ActionResult::new_system_message(
                data, 
                vec![player_id.to_string()], 
                format!("玩家{}丢弃了物品{}", player.name, item_id)
            );
            Ok(action_result)
        } else {
            Err("Item not found in player's inventory".to_string())
        }
    }

    /// 处理传音行动
    pub fn handle_deliver_action(&mut self, player_id: &str, target_player_id: &str, message: &str) -> Result<ActionResult, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态
        if !player.is_alive {
            return Err("Player is not alive".to_string());
        }
        
        // 消耗体力值（根据规则配置，假设传音消耗5点体力）
        let deliver_cost = 5;
        if player.strength >= deliver_cost {
            player.strength -= deliver_cost;
        } else {
            player.strength = 0;
        }
        
        // 向目标玩家发送消息
        // 在实际实现中，这里需要找到目标玩家的连接并发送消息
        // 这里我们只是构造响应
        
        let data = serde_json::json!({
            "message": format!("玩家{}向您传音: {}", player.name, message)
        });
        
        // 创建动作结果，广播给发送者和接收者
        let action_result = ActionResult::new_user_message(
            data, 
            vec![player_id.to_string(), target_player_id.to_string()], 
            format!("玩家{}向玩家{}发送消息: {}", player.name, target_player_id, message)
        );
        Ok(action_result)
    }

    /// 处理发送消息给导演行动
    pub fn handle_send_action(&mut self, player_id: &str, message: &str) -> Result<ActionResult, String> {
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 将消息转发给导演客户端
        // 在实际实现中，这里需要找到导演的连接并发送消息
        // 这里我们只是构造响应
        
        let data = serde_json::json!({
            "message": format!("玩家{}向导演发送消息: {}", player.name, message)
        });
        
        // 创建动作结果，只广播给发起者本人（导演会收到所有消息）
        let action_result = ActionResult::new_user_message(
            data, 
            vec![player_id.to_string()], 
            format!("玩家{}向导演发送消息: {}", player.name, message)
        );
        Ok(action_result)
    }
}