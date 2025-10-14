//! 玩家行动调度器
//! 
//! 负责玩家行动的权限验证和分发调度

use super::models::{GameState, ActionResult, Player};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 玩家行动参数结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionParams {
    /// 移动目标地点
    pub target_place: Option<String>,
    
    /// 出生地点名称
    pub place_name: Option<String>,
    
    /// 道具ID
    pub item_id: Option<String>,
    
    /// 装备槽位类型
    pub slot_type: Option<String>,
    
    /// 目标玩家ID
    pub target_player_id: Option<String>,
    
    /// 消息内容
    pub message: Option<String>,
}

impl ActionParams {
    /// 从JSON数据构造ActionParams
    pub fn from_json(data: &JsonValue) -> Result<Self, String> {
        serde_json::from_value(data.clone())
            .map_err(|e| format!("Failed to parse action params: {}", e))
    }
}

/// 验证类型枚举
#[derive(Debug)]
enum ValidationType {
    Alive,                                     // 玩家存活
    Born,                                      // 玩家已出生
    NotBorn,                                   // 玩家未出生
    NotBound,                                  // 玩家未被捆绑
    Strength(i32),                            // 体力验证（带消耗值）
    InventorySpace,                           // 背包空间验证
}

/// 玩家行动调度器
pub struct PlayerActionScheduler;

macro_rules! validate_or_return {
    ($game_state:expr, $player_id:expr, $validations:expr) => {
        for validation in $validations {
            if let Err(result) = PlayerActionScheduler::validate($game_state, $player_id, validation) {
                return Ok(result);
            }
        }
    };
}

impl PlayerActionScheduler {
    /// 调度玩家行动
    /// 
    /// # 参数
    /// - `game_state`: 可变游戏状态引用
    /// - `player_id`: 执行行动的玩家ID
    /// - `action_type`: 行动类型字符串
    /// - `action_params`: 行动参数
    /// 
    /// # 返回值
    /// - `Ok(ActionResult)`: 行动执行成功或验证失败
    /// - `Err(String)`: 系统级错误
    pub fn dispatch(
        game_state: &mut GameState,
        player_id: &str,
        action_type: &str,
        action_params: ActionParams,
    ) -> Result<ActionResult, String> {
        // 执行验证
        match action_type {
            "born" => {
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::NotBound, ValidationType::NotBorn]);
                let place_name = action_params.place_name.as_ref().ok_or("Missing place_name parameter")?;
                return game_state.handle_born_action(player_id, place_name);
            },
            "move" => {
                let move_cost = game_state.rule_engine.action_costs.move_cost;
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound,
                     ValidationType::Strength(move_cost)]);
                let target_place = action_params.target_place.as_ref().ok_or("Missing target_place parameter")?;
                return game_state.handle_move_action(player_id, target_place);
            },
            "search" => {
                let search_cost = game_state.rule_engine.action_costs.search;
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound,
                     ValidationType::Strength(search_cost)]);
                return game_state.handle_search_action(player_id);
            },
            "pick" => {
                let pick_cost = game_state.rule_engine.action_costs.pick;
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound,
                     ValidationType::InventorySpace,
                     ValidationType::Strength(pick_cost)]);
                return game_state.handle_pick_action(player_id);
            },
            "attack" => {
                let attack_cost = game_state.rule_engine.action_costs.attack;
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound,
                     ValidationType::Strength(attack_cost)]);
                let target_player_id = action_params.target_player_id.as_ref().ok_or("Missing target_player_id parameter")?;
                return game_state.handle_attack_action(player_id, target_player_id);
            },
            "equip" => {
                let equip_cost = game_state.rule_engine.action_costs.equip;
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound,
                     ValidationType::Strength(equip_cost)]);
                let item_id = action_params.item_id.as_ref().ok_or("Missing item_id parameter")?;
                return game_state.handle_equip_action(player_id, item_id);
            },
            "use" => {
                let use_cost = game_state.rule_engine.action_costs.use_item;
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound,
                     ValidationType::Strength(use_cost)]);
                let item_id = action_params.item_id.as_ref().ok_or("Missing item_id parameter")?;
                return game_state.handle_use_action(player_id, item_id);
            },
            "throw" => {
                let throw_cost = game_state.rule_engine.action_costs.throw_item;
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound,
                     ValidationType::Strength(throw_cost)]);
                let item_id = action_params.item_id.as_ref().ok_or("Missing item_id parameter")?;
                return game_state.handle_throw_action(player_id, item_id);
            },
            "unequip" => {
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound]);
                let slot_type = action_params.slot_type.as_ref().ok_or("Missing slot_type parameter")?;
                return game_state.handle_unequip_action(player_id, slot_type);
            },
            "deliver" => {
                let deliver_cost = game_state.rule_engine.action_costs.deliver;
                validate_or_return!(game_state, player_id, vec![ValidationType::Alive, ValidationType::Born, ValidationType::NotBound,
                     ValidationType::Strength(deliver_cost)]);
                let target_player_id = action_params.target_player_id.as_ref().ok_or("Missing target_player_id parameter")?;
                let message = action_params.message.as_ref().ok_or("Missing message parameter")?;
                return game_state.handle_deliver_action(player_id, target_player_id, message);
            },
            "send" => {
                let message = action_params.message.as_ref().ok_or("Missing message parameter")?;
                return game_state.handle_send_to_director_action(player_id, message);
            },
            _ => return Err(format!("Unknown action type: {}", action_type)),
        };
    }

    /// 执行单个验证
    fn validate(game_state: &GameState, player_id: &str, validation: ValidationType) -> Result<(), ActionResult> {
        // 首先检查玩家是否存在
        let player = match game_state.players.get(player_id) {
            Some(player) => player,
            None => {
                let data = serde_json::json!({});
                return Err(ActionResult::new_info_message(
                    data,
                    vec![player_id.to_string()],
                    "玩家未找到".to_string(),
                    false
                ));
            }
        };
        
        // 玩家存在，现在使用玩家引用来进行具体验证
        match validation {
            ValidationType::Alive => Self::check_player_alive_from_ref(player, player_id),
            ValidationType::Born => Self::check_player_born_from_ref(player, player_id),
            ValidationType::NotBorn => Self::check_player_not_born_from_ref(player, player_id),
            ValidationType::NotBound => Self::check_player_not_bound_from_ref(player, player_id),
            ValidationType::Strength(required) => Self::check_player_strength_from_ref(player, player_id, required),
            ValidationType::InventorySpace => Self::check_inventory_space_from_ref(player, game_state, player_id),
        }
    }
    
    // ==================== 权限验证辅助函数 ====================
    
    /// 验证玩家是否存活（从玩家引用）
    fn check_player_alive_from_ref(player: &Player, player_id: &str) -> Result<(), ActionResult> {
        if !player.is_alive {
            let data = serde_json::json!({});
            return Err(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "玩家已死亡，无法进行操作".to_string(),
                false
            ));
        }
        Ok(())
    }
    
    /// 验证玩家是否已出生（从玩家引用）
    fn check_player_born_from_ref(player: &Player, player_id: &str) -> Result<(), ActionResult> {
        if player.location.is_empty() {
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
    
    /// 验证玩家尚未出生（从玩家引用）
    fn check_player_not_born_from_ref(player: &Player, player_id: &str) -> Result<(), ActionResult> {
        if !player.location.is_empty() {
            let data = serde_json::json!({});
            return Err(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "玩家已经出生，无法重复出生".to_string(),
                false
            ));
        }
        Ok(())
    }
    
    /// 验证玩家体力是否充足（从玩家引用）
    fn check_player_strength_from_ref(player: &Player, player_id: &str, required: i32) -> Result<(), ActionResult> {
        if player.strength < required {
            let data = serde_json::json!({});
            return Err(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "体力不足，无法执行该操作".to_string(),
                false
            ));
        }
        Ok(())
    }
    
    /// 验证玩家未被捆绑（从玩家引用）
    fn check_player_not_bound_from_ref(player: &Player, player_id: &str) -> Result<(), ActionResult> {
        if player.is_bound {
            let data = serde_json::json!({});
            return Err(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "玩家被捆绑，无法自由行动".to_string(),
                false
            ));
        }
        Ok(())
    }
    
    /// 验证背包有空闲空间（从玩家引用）
    fn check_inventory_space_from_ref(player: &Player, game_state: &GameState, player_id: &str) -> Result<(), ActionResult> {
        // 获取背包最大容量
        let max_inventory_size = game_state.rule_engine.player_config.max_backpack_items as usize;
        
        // 检查当前物品总数（包括装备）
        let total_items = player.get_total_item_count();
        
        if total_items >= max_inventory_size {
            let data = serde_json::json!({});
            return Err(ActionResult::new_info_message(
                data,
                vec![player_id.to_string()],
                "背包已满，无法拾取更多物品".to_string(),
                false
            ));
        }
        Ok(())
    }
}
