//! GameState 导演夜晚结束管理逻辑实现
//! 导演在前端手动点击夜晚结束结算按钮，进行单次夜晚结束的结算：
//! 缩圈逻辑：调用handle_modify_place，依次遍历导演设置的本轮缩圈地点`next_night_destroyed_places`，如果有未被摧毁的，摧毁之（自动触发缩圈杀死玩家的逻辑）。
//! 持续流血：所有身上bleed_damage大于0的玩家受到一次流血伤害；如果此次流血导致死亡，结算杀死玩家`kill_player`事件（无击杀者）。
//! 加体力逻辑：为所有存活玩家增加规则设定的每日自动恢复体力值`daily_strength_recovery`。
//! 休养逻辑：如果夜晚结束时`rest_mode`仍然为真，恢复规则设置中规定的生命值，并将体力恢复至满值。
//! 每日清除：清空`next_night_destroyed_places`。调用daily_reset()清除玩家状态。

use crate::websocket::models::{GameState, ActionResult, ActionResults, AirdropItem, ItemDeletionItem};

impl GameState {
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
}