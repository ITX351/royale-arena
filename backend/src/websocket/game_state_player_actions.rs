//! GameState 玩家行动实现

use super::models::*;

impl GameState {
    /// 检查玩家基础状态（存活状态和体力值）
    /// 
    /// # 参数
    /// - `player_id`: 玩家ID
    /// - `required_strength`: 所需的最少体力值，如果为None则不检查体力
    /// 
    /// # 返回值
    /// - `Ok(())`: 状态检查通过
    /// - `Err(ActionResult)`: 状态检查失败，返回Info类型的错误消息
    fn check_player_basic_status(&self, player_id: &str, required_strength: Option<i32>) -> Result<(), ActionResult> {
        let player = self.players.get(player_id)
            .ok_or_else(|| {
                let data = serde_json::json!({});
                ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    "玩家未找到".to_string(),
                    false
                )
            })?;
        
        // 检查玩家是否存活
        if !player.is_alive {
            let data = serde_json::json!({});
            return Err(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "玩家已死亡，无法进行操作".to_string(),
                false
            ));
        }
        
        // 检查体力值（如果指定了最低要求）
        if let Some(min_strength) = required_strength {
            if player.strength < min_strength {
                let data = serde_json::json!({});
                return Err(ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    "体力不足，无法执行该操作".to_string(),
                    false
                ));
            }
        }
        
        Ok(())
    }

    /// 检查玩家是否已出生
    /// 
    /// # 参数
    /// - `player_id`: 玩家ID
    /// 
    /// # 返回值
    /// - `Ok(())`: 玩家已出生，检查通过
    /// - `Err(ActionResult)`: 玩家尚未出生，返回Info类型的错误消息
    fn check_player_born_status(&self, player_id: &str) -> Result<(), ActionResult> {
        let player = self.players.get(player_id)
            .ok_or_else(|| {
                let data = serde_json::json!({});
                ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    "玩家未找到".to_string(),
                    false
                )
            })?;
        
        if !player.is_born || player.location.is_empty() {
            let data = serde_json::json!({});
            return Err(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "玩家尚未出生，请先选择出生地点".to_string(),
                false
            ));
        }
        
        Ok(())
    }
    /// 处理玩家出生行动
    pub fn handle_born_action(&mut self, player_id: &str, place_name: &str) -> Result<ActionResult, String> {
        // 使用公用函数检查玩家基础状态（出生不消耗体力）
        if let Err(action_result) = self.check_player_basic_status(player_id, None) {
            return Ok(action_result);
        }
        
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查玩家是否已经执行过出生
        if player.is_born || !player.location.is_empty() {
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "玩家已经出生".to_string(),
                false
            );
            return Ok(action_result);
        }
        
        // 验证指定地点是否存在且未被摧毁
        let place = self.places.get(place_name).ok_or("Place not found".to_string())?;
        if place.is_destroyed {
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "地点已被摧毁".to_string(),
                false
            );
            return Ok(action_result);
        }
        
        // 更新玩家位置到指定地点并设置出生状态
        player.location = place_name.to_string();
        player.is_born = true;
        
        // 将玩家添加到地点的玩家列表中
        let place_mut = self.places.get_mut(place_name).unwrap();
        place_mut.players.push(player.id.clone());
        
        // 向该玩家发送位置更新结果
        let data = serde_json::json!({
            "location": place_name,
            "is_born": true
        });
        
        // 创建动作结果，只广播给发起者本人
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string()], 
            format!("{} 在地点 {} 出生", player.name, place_name)
        );
        
        Ok(action_result)
    }

    /// 处理玩家移动行动
    pub fn handle_move_action(&mut self, player_id: &str, target_place: &str) -> Result<ActionResult, String> {
        // 使用规则引擎获取移动消耗
        let move_cost = self.rule_engine.action_costs.move_cost;
        
        // 使用公用函数检查玩家基础状态
        if let Err(action_result) = self.check_player_basic_status(player_id, Some(move_cost)) {
            return Ok(action_result);
        }
        
        // 检查玩家是否已出生
        if let Err(action_result) = self.check_player_born_status(player_id) {
            return Ok(action_result);
        }
        
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 验证目标地点是否存在且未被摧毁
        let place = self.places.get(target_place).ok_or("Target place not found".to_string())?;
        if place.is_destroyed {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "目标地点已被摧毁".to_string(),
                false
            );
            return Ok(action_result);
        }
        
        // 消耗体力值
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
            format!("{} 移动到地点 {}", player.name, target_place)
        );
        
        Ok(action_result)
    }

    /// 处理搜索行动（优化版本）
    pub fn handle_search_action(&mut self, player_id: &str) -> Result<ActionResult, String> {
        // 使用规则引擎获取搜索消耗
        let search_cost = self.rule_engine.action_costs.search;
        
        // 使用公用函数检查玩家基础状态
        if let Err(action_result) = self.check_player_basic_status(player_id, Some(search_cost)) {
            return Ok(action_result);
        }
        
        // 检查玩家是否已出生
        if let Err(action_result) = self.check_player_born_status(player_id) {
            return Ok(action_result);
        }
        
        // 获取玩家状态信息（避免借用冲突）
        let (player_location, player_last_search_time) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            (player.location.clone(), player.last_search_time)
        };
        
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
                    false
                );
                return Ok(action_result);
            }
        }
        
        // 更新玩家状态
        {
            let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            player.strength -= search_cost;
            player.last_search_time = Some(chrono::Utc::now());
        }
        
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
    pub fn handle_pick_action(&mut self, player_id: &str) -> Result<ActionResult, String> {
        // 使用规则引擎获取拾取消耗
        let pick_cost = self.rule_engine.action_costs.pick;
        
        // 使用公用函数检查玩家基础状态
        if let Err(action_result) = self.check_player_basic_status(player_id, Some(pick_cost)) {
            return Ok(action_result);
        }
        
        // 检查玩家是否已出生
        if let Err(action_result) = self.check_player_born_status(player_id) {
            return Ok(action_result);
        }
        
        // 使用规则引擎检查背包容量
        {
            let player = self.players.get(player_id).ok_or("玩家未找到".to_string())?;
            let max_backpack_items = self.rule_engine.player_config.max_backpack_items as usize;
            
            if player.inventory.len() >= max_backpack_items {
                // 背包已满，返回Info提示
                let data = serde_json::json!({
                    "message": "背包已满，无法拾取更多物品"
                });
                let action_result = ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    format!("玩家 {} 尝试拾取物品但背包已满", player.name),
                    false // 不向导演广播
                );
                return Ok(action_result);
            }
        }
        
        // 检查上一次搜索结果是否为物品
        {
            let player = self.players.get(player_id).ok_or("玩家未找到".to_string())?;
            
            let last_search_result_valid = if let Some(ref search_result) = player.last_search_result {
                search_result.target_type == SearchResultType::Item
            } else {
                false
            };
            
            if !last_search_result_valid {
                // 用Info类型返回错误提示
                let data = serde_json::json!({});
                let action_result = ActionResult::new_info_message(
                    data, 
                    vec![player_id.to_string()], 
                    "上一次搜索结果不是物品".to_string(),
                    false
                );
                return Ok(action_result);
            }
        }
        
        // 获取搜索结果信息和玩家位置
        let (player_last_search_result, player_location) = {
            let player = self.players.get(player_id).ok_or("玩家未找到".to_string())?;
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
                let data = {
                    let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
                    // 消耗体力值
                    player.strength -= pick_cost;
                    player.inventory.push(item);
                    // 清除捡拾者的上一次搜索结果，防止连续捡拾同一物品
                    player.last_search_result = None;
                    
                    // 返回更新后的玩家信息
                    serde_json::json!({
                        "inventory": player.inventory,
                        "strength": player.strength
                    })
                };
                
                // 创建动作结果，只广播给发起者本人
                let action_result = ActionResult::new_system_message(
                    data, 
                    vec![player_id.to_string()], 
                    format!("玩家 {} 捡起了一个物品 {}", 
                        self.players.get(player_id).unwrap().name,
                        item_name
                    )
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
                    format!("玩家 {} 试图捡起一个物品但该物品已不存在", 
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
        // 使用规则引擎获取攻击消耗
        let attack_cost = self.rule_engine.action_costs.attack;
        
        // 使用公用函数检查玩家基础状态
        if let Err(action_result) = self.check_player_basic_status(player_id, Some(attack_cost)) {
            return Ok(action_result);
        }
        
        // 检查前置条件：上一次搜索结果为玩家
        let (has_weapons, last_search_result_valid) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            
            let search_valid = if let Some(ref search_result) = player.last_search_result {
                search_result.target_type == SearchResultType::Player
            } else {
                false
            };
            
            (!player.equipped_weapons.is_empty(), search_valid)
        };
        
        if !last_search_result_valid {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "上一次搜索结果不是玩家".to_string(),
                false
            );
            return Ok(action_result);
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
        let (target_player_location, target_player_alive, target_player_name) = {
            let target_player = self.players.get(&target_player_id).ok_or("Target player not found".to_string())?;
            (target_player.location.clone(), target_player.is_alive, target_player.name.clone())
        };
        
        // 验证目标玩家是否在同一地点
        if target_player_location != player_location {
            // 目标玩家已离开
            let data = serde_json::json!({
                "message": "Target player has left the location"
            });
            
            // 用Info类型返回错误提示
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "目标玩家已离开该地点".to_string(),
                false
            );
            return Ok(action_result);
        }
        
        // 验证目标玩家是否已死亡
        if !target_player_alive {
            // 目标玩家已死亡，不能攻击
            let data = serde_json::json!({
                "message": "Target player is already dead"
            });
            
            // 用Info类型返回错误提示
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "目标玩家已死亡".to_string(),
                false
            );
            return Ok(action_result);
        }
        
        // 根据是否装备武器计算伤害
        let (damage, attack_method) = if has_weapons {
            // 有武器：使用武器伤害（简化处理）
            (20, "武器攻击")
        } else {
            // 无武器：使用规则引擎获取挥拳伤害
            (self.rule_engine.get_unarmed_damage(), "挥拳攻击")
        };
        
        // 减少目标玩家生命值
        let was_killed = {
            let target_player = self.players.get_mut(&target_player_id).ok_or("Target player not found".to_string())?;
            target_player.life -= damage;
            
            // 检查目标玩家是否死亡
            if target_player.life <= 0 {
                target_player.life = 0;
                target_player.is_alive = false;
                true
            } else {
                false
            }
        }; // 释放对目标玩家的可变借用
        
        // 如果目标玩家被击杀，处理掉落物品
        let loot_effects = if was_killed {
            self.handle_player_death_loot(Some(player_id), &target_player_id)
        } else {
            Vec::new()
        };
        
        // 获取目标玩家的当前状态
        let (target_player_life, target_player_is_alive) = {
            let target_player = self.players.get(&target_player_id).ok_or("Target player not found".to_string())?;
            (target_player.life, target_player.is_alive)
        };

        // 向攻击者发送攻击结果（仅包括主目标）
        let data = serde_json::json!({
            "message": format!("Attacked player {} for {} damage using {}", target_player_name, damage, attack_method),
            "target_player_life": target_player_life,
            "target_player_is_alive": target_player_is_alive,
            "attack_method": attack_method,
            "damage": damage,
            "loot_effects": loot_effects // 添加掉落效果信息
        });
        
        // 向被攻击者发送被攻击通知（不包括攻击者身份）
        let _target_data = serde_json::json!({
            "message": format!("You were attacked for {} damage", damage)
        });
        
        // 消耗体力值并清除上一次搜索结果，防止连续攻击同一目标
        {
            let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            if player.strength >= attack_cost {
                player.strength -= attack_cost;
            } else {
                player.strength = 0;
            }
            // 清除攻击者的上一次搜索结果，防止连续攻击同一目标
            player.last_search_result = None;
        } // 释放对攻击者的可变借用

        // 创建动作结果，广播给攻击者和被攻击者
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string(), target_player_id.clone()], 
            format!("玩家 {} 使用{}攻击玩家 {} 造成 {} 点伤害", 
                self.players.get(player_id).unwrap().name, attack_method, target_player_name, damage)
        );
        
        Ok(action_result)
    }

    /// 处理装备行动
    pub fn handle_equip_action(&mut self, player_id: &str, item_id: &str) -> Result<ActionResult, String> {
        // 使用规则引擎获取装备消耗
        let equip_cost = self.rule_engine.action_costs.equip;
        
        // 使用公用函数检查玩家基础状态
        if let Err(action_result) = self.check_player_basic_status(player_id, Some(equip_cost)) {
            return Ok(action_result);
        }
        
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        // 检查前置条件：玩家处于存活状态，背包中有指定物品
        if !player.is_alive {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "玩家已死亡，无法进行操作".to_string(),
                false
            );
            return Ok(action_result);
        }

        // 验证玩家背包中是否存在指定物品
        if let Some(_item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            // 消耗体力值
            player.strength -= equip_cost;
            
            // 更新玩家当前手持物品
            player.hand_item = Some(item_id.to_string());
            
            // 向该玩家发送手持物品状态更新
            let data = serde_json::json!({
                "hand_item": item_id,
                "strength": player.strength
            });
            
            // 创建动作结果，只广播给发起者本人
            let action_result = ActionResult::new_system_message(
                data, 
                vec![player_id.to_string()], 
                format!("玩家 {} 装备了物品 {}", player.name, item_id)
            );
            Ok(action_result)
        } else {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "背包中没有该道具".to_string(),
                false
            );
            return Ok(action_result);
        }
    }

    /// 处理使用道具行动
    pub fn handle_use_action(&mut self, player_id: &str, item_id: &str) -> Result<ActionResult, String> {
        // 使用规则引擎获取使用消耗
        let use_cost = self.rule_engine.action_costs.use_item;
        
        // 使用公用函数检查玩家基础状态
        if let Err(action_result) = self.check_player_basic_status(player_id, Some(use_cost)) {
            return Ok(action_result);
        }
        
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
        
        if player.hand_item.is_none() {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "当前没有手持道具".to_string(),
                false
            );
            return Ok(action_result);
        }
        
        // 验证手持的是否是指定物品
        if player.hand_item.as_ref() != Some(&item_id.to_string()) {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "指定的道具不是当前手持道具".to_string(),
                false
            );
            return Ok(action_result);
        }

        // 查找物品信息
        if let Some(item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            let item = &player.inventory[item_index];
            
            // 消耗体力值
            player.strength -= use_cost;
            
            // 根据道具类型执行相应效果
            match item.item_type {
                ItemType::Consumable => {
                    // 如果是消耗品：恢复生命值、传送等，并从玩家背包中移除消耗品
                    // 这里我们简化处理，假设所有消耗品都恢复20点生命值
                    player.life += 20;
                    if player.life > player.max_life {
                        player.life = player.max_life; // 使用玩家的最大生命值
                    }
                    
                    // 从玩家背包中移除消耗品
                    player.inventory.remove(item_index);
                    
                    // 清空手持物品
                    player.hand_item = None;
                    
                    // 更新玩家状态
                    let data = serde_json::json!({
                        "life": player.life,
                        "inventory": player.inventory,
                        "hand_item": null,
                        "strength": player.strength
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家 {} 使用了消耗品 {}", player.name, item_id)
                    );
                    Ok(action_result)
                }
                ItemType::Weapon => {
                    // 如果是装备类：装备到对应位置
                    match item.item_type {
                        ItemType::Weapon => {
                            if player.can_equip_weapon(&self.rule_engine) {
                                player.equip_weapon(item_id.to_string(), item.clone());
                            }
                        }
                        ItemType::Equipment => {
                            if player.can_equip_armor(&self.rule_engine) {
                                player.equip_armor(item_id.to_string(), item.clone());
                            }
                        }
                        _ => {}
                    }
                    
                    // 更新玩家状态
                    let data = serde_json::json!({
                        "equipped_weapons": player.equipped_weapons,
                        "equipped_armors": player.equipped_armors,
                        "strength": player.strength
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家 {} 装备了武器 {}", player.name, item_id)
                    );
                    Ok(action_result)
                }
                ItemType::Equipment => {
                    // 其他装备类型处理
                    if player.can_equip_armor(&self.rule_engine) {
                        player.equip_armor(item_id.to_string(), item.clone());
                    }
                    
                    // 更新玩家状态
                    let data = serde_json::json!({
                        "equipped_armors": player.equipped_armors,
                        "strength": player.strength
                    });
                    
                    // 创建动作结果，只广播给发起者本人
                    let action_result = ActionResult::new_system_message(
                        data, 
                        vec![player_id.to_string()], 
                        format!("玩家 {} 装备了物品 {}", player.name, item_id)
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
        // 使用规则引擎获取丢弃消耗
        let throw_cost = self.rule_engine.action_costs.throw_item;
        
        // 使用公用函数检查玩家基础状态
        if let Err(action_result) = self.check_player_basic_status(player_id, Some(throw_cost)) {
            return Ok(action_result);
        }
        
        // 获取玩家引用
        let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;

        // 检查前置条件：玩家处于存活状态，背包中有指定物品
        if !player.is_alive {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "玩家已死亡，无法进行操作".to_string(),
                false
            );
            return Ok(action_result);
        }

        // 验证玩家背包中是否存在指定物品
        if let Some(item_index) = player.inventory.iter().position(|item| item.id == item_id) {
            // 消耗体力值
            player.strength -= throw_cost;
            
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
                "hand_item": player.hand_item,
                "strength": player.strength
            });
            
            // 创建动作结果，只广播给发起者本人
            let action_result = ActionResult::new_system_message(
                data, 
                vec![player_id.to_string()], 
                format!("玩家 {} 丢弃了物品 {}", player.name, item_id)
            );
            Ok(action_result)
        } else {
            // 用Info类型返回错误提示
            let data = serde_json::json!({});
            let action_result = ActionResult::new_info_message(
                data, 
                vec![player_id.to_string()], 
                "背包中没有该道具".to_string(),
                false
            );
            return Ok(action_result);
        }

    }

    /// 处理传音行动
    pub fn handle_deliver_action(&mut self, player_id: &str, target_player_id: &str, message: &str) -> Result<ActionResult, String> {
        // 使用规则引擎获取传音消耗
        let deliver_cost = self.rule_engine.action_costs.deliver;
        
        // 使用公用函数检查玩家基础状态
        if let Err(action_result) = self.check_player_basic_status(player_id, Some(deliver_cost)) {
            return Ok(action_result);
        }
        
        // 获取发送玩家和目标玩家信息
        let target_player_name = self.players.get(target_player_id).ok_or("Target player not found".to_string())?.name.clone();
        let sender_player_name = {
            // 获取发送玩家引用并消耗体力值
            let sender_player = self.players.get_mut(player_id).ok_or("Sender player not found".to_string())?;
            sender_player.strength -= deliver_cost;
            sender_player.name.clone()
        };

        // 向目标玩家发送消息
        // 在实际实现中，这里需要找到目标玩家的连接并发送消息
        // 这里我们只是构造响应

        let formatted_message = format!("玩家 {} 向玩家 {} 发送消息: {}", sender_player_name, target_player_name, message);
        
        let data = serde_json::json!({
            "message": formatted_message,
        });
        
        // 创建动作结果，广播给发送者和接收者
        let action_result = ActionResult::new_user_message(
            data, 
            vec![player_id.to_string(), target_player_id.to_string()], 
            formatted_message
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
            "message": format!("玩家 {} 向导演发送消息: {}", player.name, message)
        });
        
        // 创建动作结果，只广播给发起者本人（导演会收到所有消息）
        let action_result = ActionResult::new_user_message(
            data, 
            vec![player_id.to_string()], 
            format!("玩家 {} 向导演发送消息: {}", player.name, message)
        );
        Ok(action_result)
    }
    
    /// 汇总当前地点的所有搜索目标
    fn collect_search_targets(&self, player_id: &str) -> Vec<SearchTarget> {
        let mut targets = Vec::new();
        
        // 获取玩家当前位置
        let player_location = &self.players[player_id].location;
        
        if let Some(place) = self.places.get(player_location) {
            // 添加其他玩家到搜索目标
            for other_player_id in &place.players {
                if other_player_id != player_id {
                    // 只搜索存活的玩家
                    if let Some(other_player) = self.players.get(other_player_id) {
                        if other_player.is_alive {
                            targets.push(SearchTarget::Player(other_player_id.clone()));
                        }
                    }
                }
            }
            
            // 添加物品到搜索目标
            for item in &place.items {
                targets.push(SearchTarget::Item(item.id.clone()));
            }
        }
        
        targets
    }
    
    /// 等概率随机选择一个搜索目标
    fn select_random_target(&self, targets: &[SearchTarget]) -> SearchTarget {
        use rand::Rng;
        let mut rng = rand::rng();
        let index = rng.random_range(0..targets.len());
        targets[index].clone()
    }
    
    /// 处理空搜索结果
    fn handle_empty_search_result(&mut self, player_id: &str) -> Result<ActionResult, String> {
        {
            let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            player.last_search_result = None;
        }
        
        let (player_strength, player_last_search_time) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            (player.strength, player.last_search_time)
        };
        
        let data = serde_json::json!({
            "last_search_result": null,
            "strength": player_strength,
            "last_search_time": player_last_search_time
        });
        
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string()], 
            format!("{} 搜索但没有发现任何东西", 
                self.players.get(player_id).unwrap().name)
        );
        Ok(action_result)
    }
    
    /// 处理玩家搜索结果
    fn handle_player_search_result(&mut self, player_id: &str, target_player_id: &str, is_visible: bool) -> Result<ActionResult, String> {
        let target_player_name = {
            if let Some(target_player) = self.players.get(target_player_id) {
                target_player.name.clone()
            } else {
                return Err("Target player not found".to_string());
            }
        };
        
        // 更新玩家的上次搜索结果
        {
            let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            player.last_search_result = Some(SearchResult {
                target_type: SearchResultType::Player,
                target_id: target_player_id.to_string(),
                target_name: target_player_name.clone(),
                is_visible,
            });
        }
        
        let (player_strength, player_last_search_time) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            (player.strength, player.last_search_time)
        };
        
        let data = serde_json::json!({
            "last_search_result": {
                "target_type": "player",
                "target_id": target_player_id,
                "target_name": target_player_name,
                "is_visible": is_visible
            },
            "strength": player_strength,
            "last_search_time": player_last_search_time
        });
        
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string()], 
            format!("{} 搜索发现了玩家 {}", 
                self.players.get(player_id).unwrap().name, target_player_name)
        );
        
        Ok(action_result)
    }
    
    /// 处理物品搜索结果
    fn handle_item_search_result(&mut self, player_id: &str, item_id: &str, is_visible: bool) -> Result<ActionResult, String> {
        let (player_location, item_name) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            let location = player.location.clone();
            
            // 查找物品名称
            let name = if let Some(place) = self.places.get(&location) {
                if let Some(item) = place.items.iter().find(|item| item.id == item_id) {
                    item.name.clone()
                } else {
                    return Err("Item not found in place".to_string());
                }
            } else {
                return Err("Place not found".to_string());
            };
            
            (location, name)
        };
        
        // 更新玩家的上次搜索结果
        {
            let player = self.players.get_mut(player_id).ok_or("Player not found".to_string())?;
            player.last_search_result = Some(SearchResult {
                target_type: SearchResultType::Item,
                target_id: item_id.to_string(),
                target_name: item_name.clone(),
                is_visible,
            });
        }
        
        let (player_strength, player_last_search_time) = {
            let player = self.players.get(player_id).ok_or("Player not found".to_string())?;
            (player.strength, player.last_search_time)
        };
        
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
        
        let action_result = ActionResult::new_system_message(
            data, 
            vec![player_id.to_string()], 
            format!("{} 搜索并发现了物品 {}", 
                self.players.get(player_id).unwrap().name, item_name)
        );
        
        Ok(action_result)
    }
    
    /// 处理玩家死亡后的物品分配
    pub fn handle_player_death_loot(&mut self, killer_id: Option<&str>, dead_player_id: &str) -> Vec<String> {
        let mut effects = Vec::new();
        
        // 获取死者的所有物品和位置（避免借用冲突）
        let (dead_player_location, dead_player_items) = {
            let dead_player = &self.players[dead_player_id];
            let mut all_items = dead_player.inventory.clone();
            
            // 添加装备的物品到掉落列表
            for weapon_id in &dead_player.equipped_weapons {
                if let Some(weapon) = dead_player.equipped_items_detail.get(weapon_id) {
                    all_items.push(weapon.clone());
                }
            }
            for armor_id in &dead_player.equipped_armors {
                if let Some(armor) = dead_player.equipped_items_detail.get(armor_id) {
                    all_items.push(armor.clone());
                }
            }
            if let Some(hand_item_id) = &dead_player.hand_item {
                if let Some(hand_item) = dead_player.equipped_items_detail.get(hand_item_id) {
                    all_items.push(hand_item.clone());
                }
            }
            
            (dead_player.location.clone(), all_items)
        };
        
        if let Some(killer_id) = killer_id {
            // 获取击杀者信息（避免借用冲突）
            let (killer_current_inventory_size, killer_name) = {
                let killer = &self.players[killer_id];
                (killer.inventory.len(), killer.name.clone())
            };
            
            let max_backpack_size = 4; // 从规则配置获取，这里简化为固定值
            let available_slots = max_backpack_size - killer_current_inventory_size;
            
            if available_slots > 0 {
                // 随机选择可收缴的物品数量
                let items_to_take = available_slots.min(dead_player_items.len());
                
                // 随机打乱物品顺序
                let mut shuffled_items = dead_player_items.clone();
                use rand::seq::SliceRandom;
                let mut rng = rand::rng();
                shuffled_items.shuffle(&mut rng);
                
                // 击杀者获得前N个物品
                for item in shuffled_items.iter().take(items_to_take) {
                    self.players.get_mut(killer_id).unwrap().inventory.push(item.clone());
                    effects.push(format!("击杀者 {} 收缴了物品 {}", 
                        killer_name, item.name));
                }
                
                // 剩余物品掉落原地
                let remaining_items: Vec<Item> = shuffled_items.into_iter().skip(items_to_take).collect();
                self.drop_items_to_ground(&dead_player_location, remaining_items);
            } else {
                // 击杀者背包已满，所有物品掉落原地
                self.drop_items_to_ground(&dead_player_location, dead_player_items);
                effects.push("击杀者背包已满，所有物品掉落原地".to_string());
            }
        } else {
            // 无击杀者，所有物品掉落原地
            self.drop_items_to_ground(&dead_player_location, dead_player_items);
            effects.push("所有物品掉落原地".to_string());
        }
        
        // 清空死者的装备和背包
        if let Some(dead_player) = self.players.get_mut(dead_player_id) {
            dead_player.inventory.clear();
            dead_player.equipped_weapons.clear();
            dead_player.equipped_armors.clear();
            dead_player.equipped_items_detail.clear();
            dead_player.hand_item = None;
        }
        
        effects
    }
    
    /// 将物品掉落到指定地点
    fn drop_items_to_ground(&mut self, location: &str, items: Vec<Item>) {
        if let Some(place) = self.places.get_mut(location) {
            place.items.extend(items);
        }
    }
}