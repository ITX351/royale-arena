//! GameState 导演夜晚结束管理逻辑实现
//! 导演在前端手动点击夜晚结束结算按钮，进行单次夜晚结束的结算：
//! 缩圈逻辑：调用handle_modify_place，依次遍历导演设置的本轮缩圈地点`next_night_destroyed_places`，如果有未被摧毁的，摧毁之（自动触发缩圈杀死玩家的逻辑）。
//! 持续流血：所有身上bleed_damage大于0的玩家受到一次流血伤害；如果此次流血导致死亡，结算杀死玩家`kill_player`事件（无击杀者）。
//! 加体力逻辑：为所有存活玩家执行每日自动恢复，生命值`daily_health_recovery`，`daily_strength_recovery`。
//! 每日清除：清空`next_night_destroyed_places`。调用daily_reset()清除玩家状态。
//! 休养逻辑：如果夜晚结束时`rest_mode`仍然为真，额外恢复规则`rest_mode`中设置的生命值`life_recovery`，体力值`strength_recovery`。
//! 休养备注：（除向导演发送消息外）玩家进行任何移动以外的行动时，将`rest_mode`置为假；玩家进行移动时累加`rest_moves_used`，当`rest_moves_used`超过规则设置中的`rest_mode.max_moves`时，将`rest_mode`置为假。

use crate::websocket::actions::utils::format_delta;
use crate::websocket::models::{ActionResult, ActionResults, GameState};

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

        // 如果被标记为已摧毁，从 next_night_destroyed_places 中移除该地点（避免重复处理）
        if is_destroyed {
            self.next_night_destroyed_places.retain(|p| p != place_name);
        }

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

    /// 夜晚结算：缩圈、流血、恢复、休养与每日重置
    pub fn handle_night_settlement(&mut self) -> Result<ActionResults, String> {
        let mut results: Vec<ActionResult> = Vec::new();

        // ===== 缩圈处理 =====
        let planned_destroys = self.next_night_destroyed_places.clone();
        let mut destroyed_places: Vec<String> = Vec::new();
        let mut missing_places: Vec<String> = Vec::new();

        for place_name in planned_destroys {
            match self.places.get(&place_name).map(|p| p.is_destroyed) {
                Some(true) => {
                    // 已经摧毁，无需重复处理
                }
                Some(false) => {
                    destroyed_places.push(place_name.clone());
                    let mut place_results = self.handle_modify_place(&place_name, true)?;
                    results.append(&mut place_results.results);
                }
                None => missing_places.push(place_name),
            }
        }

        if !missing_places.is_empty() {
            let missing_joined = missing_places.join("、");
            let data = serde_json::json!({
                "missing_places": missing_places.clone(),
            });
            results.push(ActionResult::new_info_message(
                data,
                vec![],
                format!("夜晚结算：以下缩圈地点未找到，已跳过：{}", missing_joined),
                true,
            ));
        }

        // ===== 持续流血处理 =====
        let mut bleed_victim_names: Vec<String> = Vec::new();
        let mut pending_bleed_deaths: Vec<(String, String)> = Vec::new();

        for (player_id, player) in self.players.iter_mut() {
            if !player.is_alive {
                continue;
            }

            let bleed_damage = player.bleed_damage;
            if bleed_damage <= 0 {
                continue;
            }

            let player_name = player.name.clone();
            let before_life = player.life;
            let after_life = (before_life - bleed_damage).max(0);

            if after_life == before_life {
                continue;
            }

            player.life = after_life;
            bleed_victim_names.push(player_name.clone());

            let mut log_message = format!(
                "{} 因流血受到 {} 点伤害，生命值 {} ({})",
                player_name,
                bleed_damage,
                after_life,
                format_delta(after_life - before_life),
            );

            if after_life <= 0 {
                log_message.push_str("，因流血死亡");
                pending_bleed_deaths.push((player_id.clone(), player_name.clone()));
            }

            let data = serde_json::json!({
                "player_id": player_id,
                "life": after_life,
                "life_delta": after_life - before_life,
                "bleed_damage": bleed_damage,
            });

            results.push(ActionResult::new_system_message(
                data,
                vec![player_id.clone()],
                log_message,
                true,
            ));
        }

        let bleed_death_names: Vec<String> = pending_bleed_deaths
            .iter()
            .map(|(_, name)| name.clone())
            .collect();

        for (player_id, _) in &pending_bleed_deaths {
            let mut death_outcome = self.kill_player(player_id, None, "流血致死")?;
            results.append(&mut death_outcome.results);
        }

        // ===== 每日恢复处理 =====
        let mut daily_recovery_names: Vec<String> = Vec::new();
        let life_recover = self.rule_engine.player_config.daily_life_recovery;
        let strength_recover = self.rule_engine.player_config.daily_strength_recovery;

        if life_recover != 0 || strength_recover != 0 {
            for (player_id, player) in self.players.iter_mut() {
                if !player.is_alive {
                    continue;
                }

                let before_life = player.life;
                let before_strength = player.strength;

                if life_recover != 0 {
                    let new_life = ((before_life as i64) + (life_recover as i64))
                        .clamp(0, player.max_life as i64) as i32;
                    player.life = new_life;
                }

                if strength_recover != 0 {
                    let new_strength = ((before_strength as i64) + (strength_recover as i64))
                        .clamp(0, player.max_strength as i64)
                        as i32;
                    player.strength = new_strength;
                }

                let life_delta = player.life - before_life;
                let strength_delta = player.strength - before_strength;

                if life_delta == 0 && strength_delta == 0 {
                    continue;
                }

                daily_recovery_names.push(player.name.clone());

                let mut segments: Vec<String> = Vec::new();
                if life_delta != 0 {
                    segments.push(format!(
                        "生命 {} ({})",
                        player.life,
                        format_delta(life_delta)
                    ));
                }
                if strength_delta != 0 {
                    segments.push(format!(
                        "体力 {} ({})",
                        player.strength,
                        format_delta(strength_delta)
                    ));
                }

                let data = serde_json::json!({
                    "player_id": player_id,
                    "life": player.life,
                    "life_delta": life_delta,
                    "strength": player.strength,
                    "strength_delta": strength_delta,
                });

                results.push(ActionResult::new_system_message(
                    data,
                    vec![player_id.clone()],
                    format!("{} 获得每日恢复：{}", player.name, segments.join("，")),
                    true,
                ));
            }
        }

        // ===== 静养加成处理 =====
        let mut rest_recovery_names: Vec<String> = Vec::new();
        let rest_life = self.rule_engine.rest_mode.life_recovery;
        let rest_strength = self.rule_engine.rest_mode.strength_recovery;

        if rest_life != 0 || rest_strength != 0 {
            for (player_id, player) in self.players.iter_mut() {
                if !player.is_alive || !player.rest_mode {
                    continue;
                }

                let before_life = player.life;
                let before_strength = player.strength;

                if rest_life != 0 {
                    let new_life = ((before_life as i64) + (rest_life as i64))
                        .clamp(0, player.max_life as i64) as i32;
                    player.life = new_life;
                }

                if rest_strength != 0 {
                    let new_strength = ((before_strength as i64) + (rest_strength as i64))
                        .clamp(0, player.max_strength as i64)
                        as i32;
                    player.strength = new_strength;
                }

                let life_delta = player.life - before_life;
                let strength_delta = player.strength - before_strength;

                if life_delta == 0 && strength_delta == 0 {
                    continue;
                }

                rest_recovery_names.push(player.name.clone());

                let mut segments: Vec<String> = Vec::new();
                if life_delta != 0 {
                    segments.push(format!(
                        "生命 {} ({})",
                        player.life,
                        format_delta(life_delta)
                    ));
                }
                if strength_delta != 0 {
                    segments.push(format!(
                        "体力 {} ({})",
                        player.strength,
                        format_delta(strength_delta)
                    ));
                }

                let data = serde_json::json!({
                    "player_id": player_id,
                    "life": player.life,
                    "life_delta": life_delta,
                    "strength": player.strength,
                    "strength_delta": strength_delta,
                });

                results.push(ActionResult::new_system_message(
                    data,
                    vec![player_id.clone()],
                    format!("{} 静养恢复：{}", player.name, segments.join("，")),
                    true,
                ));
            }
        }

        // ===== 每日清除与状态重置 =====
        let player_ids: Vec<String> = self.players.keys().cloned().collect();
        for player_id in player_ids {
            if let Some(player) = self.players.get_mut(&player_id) {
                player.daily_reset(&self.rule_engine);
            }
        }
        self.next_night_destroyed_places.clear();

        // ===== 总结输出 =====
        let destroyed_count = destroyed_places.len();
        let bleed_victim_count = bleed_victim_names.len();
        let bleed_death_count = bleed_death_names.len();
        let daily_recovery_count = daily_recovery_names.len();
        let rest_recovery_count = rest_recovery_names.len();

        let skipped_count = missing_places.len();

        let summary_data = serde_json::json!({
            "destroyed_places": destroyed_places,
            "bleed_victims": bleed_victim_names,
            "bleed_deaths": bleed_death_names,
            "daily_recovery_count": daily_recovery_count,
            "rest_recovery_count": rest_recovery_count,
            "skipped_destroy_places": missing_places,
        });

        let mut summary_message = format!(
            "夜晚结算完成：缩圈 {} 个地点，流血结算 {} 名玩家（{} 人死亡），每日恢复 {} 名玩家，静养加成 {} 名玩家",
            destroyed_count,
            bleed_victim_count,
            bleed_death_count,
            daily_recovery_count,
            rest_recovery_count,
        );

        if skipped_count > 0 {
            summary_message.push_str("；部分缩圈地点未找到，详见数据");
        }

        results.push(ActionResult::new_info_message(
            summary_data,
            vec![],
            summary_message,
            true,
        ));

        Ok(ActionResults { results })
    }
}
