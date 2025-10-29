use rand::Rng;
use serde_json::json;

use crate::game::game_rule_engine::ItemType;
use crate::websocket::models::{ActionResult, ActionResults, GameState};

enum EquipmentSlot {
    Weapon,
    Armor,
}

impl GameState {
    pub fn handle_upgrade_equipment_action(
        &mut self,
        player_id: &str,
        upgrader_item_id: &str,
        slot_type: &str,
    ) -> Result<ActionResults, String> {
        if !self.players.contains_key(player_id) {
            return Err("Player not found".to_string());
        }

        let info_result = |message: &str| -> Result<ActionResults, String> {
            let result = ActionResult::new_info_message(
                json!({}),
                vec![player_id.to_string()],
                message.to_string(),
                false,
            );
            Ok(result.as_results())
        };

        let slot = match slot_type {
            "weapon" => EquipmentSlot::Weapon,
            "armor" => EquipmentSlot::Armor,
            _ => return info_result("无效的装备栏位"),
        };

        let upgrader_index = {
            let player = self.players.get(player_id).unwrap();
            match player
                .inventory
                .iter()
                .position(|item| item.id == upgrader_item_id)
            {
                Some(index) => index,
                None => return info_result("背包中没有该升级器"),
            }
        };

        let upgrader_item = {
            let player = self.players.get(player_id).unwrap();
            player.inventory[upgrader_index].clone()
        };

        if !matches!(upgrader_item.item_type, ItemType::Upgrader) {
            return info_result("所选道具不是升级器");
        }

        let upgrader_internal = match upgrader_item.internal_name.clone() {
            Some(name) => name,
            None => return info_result("升级器缺少内部名称配置"),
        };

        let (equipped_internal, equipped_name) = match slot {
            EquipmentSlot::Weapon => {
                let player = self.players.get(player_id).unwrap();
                match player.equipped_weapon.as_ref() {
                    Some(weapon) => match weapon.internal_name.clone() {
                        Some(internal) => (internal, weapon.name.clone()),
                        None => return info_result("当前武器缺少内部名称配置"),
                    },
                    None => return info_result("当前未装备武器"),
                }
            }
            EquipmentSlot::Armor => {
                let player = self.players.get(player_id).unwrap();
                match player.equipped_armor.as_ref() {
                    Some(armor) => match armor.internal_name.clone() {
                        Some(internal) => (internal, armor.name.clone()),
                        None => return info_result("当前护甲缺少内部名称配置"),
                    },
                    None => return info_result("当前未装备护甲"),
                }
            }
        };

        let recipes = match self
            .rule_engine
            .items_config
            .upgrade_recipes
            .get(&upgrader_internal)
        {
            Some(entries) => entries,
            None => return info_result("升级器没有可用配方"),
        };

        let recipe = match recipes.iter().find(|recipe| {
            recipe
                .ingredients
                .iter()
                .any(|ingredient| ingredient == &equipped_internal)
        }) {
            Some(recipe) => recipe,
            None => return info_result("未找到匹配的合成配方"),
        };

        let candidate_names = match slot {
            EquipmentSlot::Weapon => match self
                .rule_engine
                .find_weapon_config_by_internal_name(&recipe.result)
            {
                Some(config) => config.display_names.clone(),
                None => return info_result("未找到合成结果的武器配置"),
            },
            EquipmentSlot::Armor => match self
                .rule_engine
                .find_armor_config_by_internal_name(&recipe.result)
            {
                Some(config) => config.display_names.clone(),
                None => return info_result("未找到合成结果的护甲配置"),
            },
        };

        let existing_names = self.collect_existing_item_names();
        let available_names: Vec<String> = candidate_names
            .into_iter()
            .filter(|name| !existing_names.contains(name))
            .collect();

        if available_names.is_empty() {
            return info_result("场上已存在所有该装备的显示名称，合成失败");
        }

        let new_display_name = if available_names.len() == 1 {
            available_names[0].clone()
        } else {
            let mut rng = rand::rng();
            let index = rng.random_range(0..available_names.len());
            available_names[index].clone()
        };

        let new_item = self
            .rule_engine
            .create_item_from_name(&new_display_name)
            .map_err(|err| format!("Failed to create upgraded item: {}", err))?;

        let mut pending_item = Some(new_item);
        let (
            player_name,
            upgrader_name,
            inventory_snapshot,
            equipped_weapon_snapshot,
            equipped_armor_snapshot,
        ) = {
            let player = self.players.get_mut(player_id).unwrap();
            let removed_upgrader = player.inventory.remove(upgrader_index);
            let upgrader_name = removed_upgrader.name.clone();
            let player_name = player.name.clone();

            match slot {
                EquipmentSlot::Weapon => {
                    if let Some(item) = pending_item.take() {
                        let _ = player.equipped_weapon.replace(item);
                    }
                }
                EquipmentSlot::Armor => {
                    if let Some(item) = pending_item.take() {
                        let _ = player.equipped_armor.replace(item);
                    }
                }
            }

            (
                player_name,
                upgrader_name,
                player.inventory.clone(),
                player.equipped_weapon.clone(),
                player.equipped_armor.clone(),
            )
        };

        let upgrade_cost = self.rule_engine.action_costs.use_item;
        self.consume_strength(player_id, upgrade_cost)?;
        let strength_after = self.players.get(player_id).unwrap().strength;

        let new_equipped_name = match slot {
            EquipmentSlot::Weapon => equipped_weapon_snapshot
                .as_ref()
                .map(|item| item.name.clone())
                .unwrap_or_default(),
            EquipmentSlot::Armor => equipped_armor_snapshot
                .as_ref()
                .map(|item| item.name.clone())
                .unwrap_or_default(),
        };

        let data = match slot {
            EquipmentSlot::Weapon => json!({
                "inventory": inventory_snapshot,
                "equipped_weapon": equipped_weapon_snapshot,
                "strength": strength_after
            }),
            EquipmentSlot::Armor => json!({
                "inventory": inventory_snapshot,
                "equipped_armor": equipped_armor_snapshot,
                "strength": strength_after
            }),
        };

        let log_message = match slot {
            EquipmentSlot::Weapon => format!(
                "{} 使用 {} 将 {} 升级为 {}",
                player_name, upgrader_name, equipped_name, new_equipped_name
            ),
            EquipmentSlot::Armor => format!(
                "{} 使用 {} 将 {} 升级为 {}",
                player_name, upgrader_name, equipped_name, new_equipped_name
            ),
        };

        let action_result =
            ActionResult::new_system_message(data, vec![player_id.to_string()], log_message, true);

        Ok(action_result.as_results())
    }
}
