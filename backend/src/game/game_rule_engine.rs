//! 游戏规则引擎
//! 负责解析和管理游戏规则配置，确保前后端规则一致性

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 物品实例，背包与场景中统一使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// 物品ID
    pub id: String,
    /// 物品显示名称
    pub name: String,
    /// 物品内部名称（如果存在）
    pub internal_name: Option<String>,
    /// 物品稀有度（仅武器、防具、升级器等有值）
    pub rarity: Option<String>,
    /// 物品具体类型及属性
    pub item_type: ItemType,
}

impl Item {
    /// 创建基础物品实例
    fn new(
        name: String,
        internal_name: Option<String>,
        rarity: Option<String>,
        item_type: ItemType,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            internal_name,
            rarity,
            item_type,
        }
    }

    /// 获取武器属性
    pub fn as_weapon(&self) -> Option<&WeaponProperties> {
        if let ItemType::Weapon(properties) = &self.item_type {
            Some(properties)
        } else {
            None
        }
    }

    /// 获取防具属性
    pub fn as_armor(&self) -> Option<&ArmorProperties> {
        if let ItemType::Armor(properties) = &self.item_type {
            Some(properties)
        } else {
            None
        }
    }

    /// 获取消耗品效果
    pub fn as_consumable(&self) -> Option<&ConsumableEffect> {
        if let ItemType::Consumable(effect) = &self.item_type {
            Some(effect)
        } else {
            None
        }
    }

    /// 获取工具/陷阱/升级器属性
    #[allow(dead_code)]
    pub fn as_utility(&self) -> Option<&UtilityProperties> {
        if let ItemType::Utility(properties) = &self.item_type {
            Some(properties)
        } else {
            None
        }
    }

    /// 获取升级器属性
    #[allow(dead_code)]
    pub fn as_upgrader(&self) -> Option<&UpgraderProperties> {
        if let ItemType::Upgrader(properties) = &self.item_type {
            Some(properties)
        } else {
            None
        }
    }
}

/// 物品类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "properties", rename_all = "snake_case")]
pub enum ItemType {
    Weapon(WeaponProperties),
    Armor(ArmorProperties),
    Consumable(ConsumableEffect),
    Utility(UtilityProperties),
    Upgrader(UpgraderProperties),
}

/// 游戏规则引擎
#[derive(Debug, Clone)]
pub struct GameRuleEngine {
    pub map_config: MapConfig,
    pub player_config: PlayerConfig,
    pub action_costs: ActionCosts,
    pub rest_mode: RestModeConfig,
    pub items_config: ItemsConfig,
    pub teammate_behavior: TeammateBehaior,
    pub death_item_disposition: DeathItemDisposition,
}

/// 地图配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    pub places: Vec<String>,
    pub safe_places: Vec<String>,
}

/// 玩家配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub max_life: i32,
    pub max_strength: i32,
    pub daily_strength_recovery: i32,
    pub search_cooldown: i64,
    pub max_backpack_items: usize,
    pub unarmed_damage: i32, // 挥拳伤害
}

/// 行动消耗配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCosts {
    #[serde(rename = "move")]
    pub move_cost: i32,
    pub search: i32,
    pub pick: i32,
    pub attack: i32,
    pub equip: i32,
    #[serde(rename = "use")]
    pub use_item: i32,
    #[serde(rename = "throw")]
    pub throw_item: i32,
    pub deliver: i32,
}

/// 静养模式配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestModeConfig {
    pub life_recovery: i32,
    pub max_moves: i32,
}

/// 队友行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammateBehaior {
    pub mode: i32, // 0: 无队友伤害免疫, 1: 有队友伤害免疫
}

/// 死亡物品处置规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathItemDisposition {
    pub description: String, // "由击杀者收缴（无击杀者则掉落在原地）"
}

/// 物品系统配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsConfig {
    pub rarity_levels: Vec<RarityLevel>,
    pub weapons: Vec<WeaponConfig>,
    pub armors: Vec<ArmorConfig>,
    pub other_items: Vec<OtherItemConfig>,
    pub consumables: Vec<ConsumableConfig>,
    pub upgraders: Vec<UpgraderConfig>,
    pub upgrade_recipes: HashMap<String, Vec<UpgradeRecipe>>,
}

/// 稀有度等级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityLevel {
    pub internal_name: String,
    pub display_name: String,
    pub prefix: String,
    pub is_airdropped: bool,
}

/// 武器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponConfig {
    pub internal_name: String,
    pub display_names: Vec<String>,
    pub rarity: String,
    pub properties: WeaponProperties,
}

/// 武器属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponProperties {
    pub damage: i32,
    pub uses: Option<i32>,
    pub votes: i32,
    pub aoe_damage: Option<i32>,
    pub bleed_damage: Option<i32>,
}

/// 防具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorConfig {
    pub internal_name: String,
    pub display_names: Vec<String>,
    pub rarity: String,
    pub properties: ArmorProperties,
}

/// 防具属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorProperties {
    pub defense: i32,
    pub votes: i32,
    pub uses: Option<i32>,
}

/// 其他物品配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherItemConfig {
    pub name: String,
    pub category: String,
    pub properties: OtherItemProperties,
}

/// 其他物品属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherItemProperties {
    pub votes: i32,
    pub uses: Option<i32>,
    pub targets: Option<i32>,
    pub damage: Option<i32>,
}

/// 消耗品配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableConfig {
    pub name: String,
    pub effect_type: String,
    pub effect_value: i32,
    pub cure_bleed: Option<i32>,
}

/// 消耗品效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableEffect {
    pub effect_type: String,
    pub effect_value: i32,
    pub cure_bleed: Option<i32>,
}

/// 升级器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgraderConfig {
    pub internal_name: String,
    pub display_names: Vec<String>,
    pub rarity: String,
}

/// 工具 / 陷阱属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityProperties {
    pub utility_type: String,
    pub votes: Option<i32>,
    pub uses: Option<i32>,
    pub targets: Option<i32>,
    pub damage: Option<i32>,
}

/// 升级器属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgraderProperties {
    pub upgrader_type: String,
}

/// 升级配方
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeRecipe {
    pub ingredients: Vec<String>,
    pub result: String,
}

/// 伤害计算结果
#[derive(Debug, Clone)]
pub struct DamageResult {
    pub damage: i32,
    pub aoe_damage: Option<i32>,
    pub bleed_damage: Option<i32>,
    pub is_fatal: bool,
}

impl GameRuleEngine {
    /// 从JSON配置创建规则引擎
    pub fn from_json(rules_json: &str) -> Result<Self, String> {
        let rules_value: serde_json::Value = serde_json::from_str(rules_json)
            .map_err(|e| format!("Failed to parse rules JSON: {}", e))?;

        // 解析地图配置
        let map_config = serde_json::from_value(
            rules_value
                .get("map")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
        )
        .map_err(|e| format!("Failed to parse map config: {}", e))?;

        // 解析玩家配置
        let player_config = serde_json::from_value(
            rules_value
                .get("player")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
        )
        .map_err(|e| format!("Failed to parse player config: {}", e))?;

        // 解析行动消耗配置
        let action_costs = serde_json::from_value(
            rules_value
                .get("action_costs")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
        )
        .map_err(|e| format!("Failed to parse action costs: {}", e))?;

        // 解析静养模式配置
        let rest_mode = serde_json::from_value(
            rules_value
                .get("rest_mode")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
        )
        .map_err(|e| format!("Failed to parse rest mode config: {}", e))?;

        // 解析物品系统配置
        let items_config = serde_json::from_value(
            rules_value
                .get("items")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
        )
        .map_err(|e| format!("Failed to parse items config: {}", e))?;

        // 解析队友行为配置
        let teammate_behavior = TeammateBehaior {
            mode: rules_value
                .get("teammate_behavior")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
        };

        // 解析死亡物品处置配置
        let death_item_disposition = DeathItemDisposition {
            description: rules_value
                .get("death_item_disposition")
                .and_then(|v| v.as_str())
                .unwrap_or("由击杀者收缴（无击杀者则掉落在原地）")
                .to_string(),
        };

        Ok(Self {
            map_config,
            player_config,
            action_costs,
            rest_mode,
            items_config,
            teammate_behavior,
            death_item_disposition,
        })
    }

    /// 计算武器伤害
    pub fn calculate_weapon_damage(
        &self,
        weapon_internal_name: &str,
        target_defense: i32,
    ) -> Result<DamageResult, String> {
        // 查找武器配置
        let weapon = self
            .items_config
            .weapons
            .iter()
            .find(|w| w.internal_name == weapon_internal_name)
            .ok_or_else(|| format!("Weapon not found: {}", weapon_internal_name))?;

        // 计算基础伤害
        let base_damage = weapon.properties.damage;
        let actual_damage = (base_damage - target_defense).max(0);

        Ok(DamageResult {
            damage: actual_damage,
            aoe_damage: weapon.properties.aoe_damage,
            bleed_damage: weapon.properties.bleed_damage,
            is_fatal: actual_damage > 0,
        })
    }

    /// 获取搜索冷却时间
    pub fn get_search_cooldown(&self) -> i64 {
        self.player_config.search_cooldown
    }

    /// 获取挥拳伤害
    pub fn get_unarmed_damage(&self) -> i32 {
        self.player_config.unarmed_damage
    }

    /// 检查地点是否为安全区
    pub fn is_safe_place(&self, place_name: &str) -> bool {
        self.map_config
            .safe_places
            .contains(&place_name.to_string())
    }

    /// 根据物品名称从规则JSON中查找并创建物品对象
    pub fn create_item_from_name(&self, item_name: &str) -> Result<Item, String> {
        // 1. 搜索武器
        for weapon in &self.items_config.weapons {
            for display_name in &weapon.display_names {
                if display_name == item_name {
                    return Ok(Item::new(
                        item_name.to_string(),
                        Some(weapon.internal_name.clone()),
                        Some(weapon.rarity.clone()),
                        ItemType::Weapon(weapon.properties.clone()),
                    ));
                }
            }
        }

        // 2. 搜索防具
        for armor in &self.items_config.armors {
            for display_name in &armor.display_names {
                if display_name == item_name {
                    return Ok(Item::new(
                        item_name.to_string(),
                        Some(armor.internal_name.clone()),
                        Some(armor.rarity.clone()),
                        ItemType::Armor(armor.properties.clone()),
                    ));
                }
            }
        }

        // 3. 搜索消耗品
        for consumable in &self.items_config.consumables {
            if consumable.name == item_name {
                let effect = ConsumableEffect {
                    effect_type: consumable.effect_type.clone(),
                    effect_value: consumable.effect_value,
                    cure_bleed: consumable.cure_bleed,
                };

                return Ok(Item::new(
                    item_name.to_string(),
                    None,
                    None,
                    ItemType::Consumable(effect),
                ));
            }
        }

        // 4. 搜索其他道具
        for other_item in &self.items_config.other_items {
            if other_item.name == item_name {
                let utility = UtilityProperties {
                    utility_type: other_item.category.clone(),
                    votes: Some(other_item.properties.votes),
                    uses: other_item.properties.uses,
                    targets: other_item.properties.targets,
                    damage: other_item.properties.damage,
                };

                return Ok(Item::new(
                    item_name.to_string(),
                    None,
                    None,
                    ItemType::Utility(utility),
                ));
            }
        }

        // 5. 搜索升级器
        for upgrader in &self.items_config.upgraders {
            for display_name in &upgrader.display_names {
                if display_name == item_name {
                    return Ok(Item::new(
                        item_name.to_string(),
                        Some(upgrader.internal_name.clone()),
                        Some(upgrader.rarity.clone()),
                        ItemType::Upgrader(UpgraderProperties {
                            upgrader_type: upgrader.internal_name.clone(),
                        }),
                    ));
                }
            }
        }

        // 如果没有找到匹配的物品，返回错误
        Err(format!("未在规则JSON中找到物品: {}", item_name))
    }
}
