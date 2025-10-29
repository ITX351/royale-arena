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
    #[allow(dead_code)]
    pub fn as_consumable(&self) -> Option<&ConsumableProperties> {
        if let ItemType::Consumable(properties) = &self.item_type {
            Some(properties)
        } else {
            None
        }
    }

    /// 获取工具/陷阱属性
    #[allow(dead_code)]
    pub fn as_utility(&self) -> Option<&UtilityProperties> {
        if let ItemType::Utility(properties) = &self.item_type {
            Some(properties)
        } else {
            None
        }
    }

    /// 是否为升级器
    #[allow(dead_code)]
    pub fn is_upgrader(&self) -> bool {
        matches!(self.item_type, ItemType::Upgrader)
    }

    /// 判断物品是否是武器或防具（即需要保持唯一性的物品）
    pub fn is_weapon_or_armor(&self) -> bool {
        matches!(self.item_type, ItemType::Weapon(_) | ItemType::Armor(_))
    }
}

/// 物品类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "properties", rename_all = "snake_case")]
pub enum ItemType {
    Weapon(WeaponProperties),
    Armor(ArmorProperties),
    Consumable(ConsumableProperties),
    Utility(UtilityProperties),
    Upgrader,
}

/// 游戏规则引擎
#[derive(Debug, Clone)]
pub struct GameRuleEngine {
    pub map_config: MapConfig,
    pub player_config: PlayerConfig,
    pub action_costs: ActionCosts,
    pub rest_mode: RestModeConfig,
    pub items_config: ItemsConfig,
    #[allow(dead_code)]
    pub teammate_behavior: TeammateBehavior, // TODO: 实现队友行为规则
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
    pub daily_life_recovery: i32,
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
    pub strength_recovery: i32,
    pub max_moves: i32,
}

/// 队友行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammateBehavior {
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
    #[serde(default)]
    pub rarity_levels: Vec<RarityLevel>,
    #[serde(default)]
    pub items: ItemsByCategory,
    #[serde(default)]
    pub upgrade_recipes: HashMap<String, Vec<UpgradeRecipe>>,
}

/// 物品分类集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemsByCategory {
    #[serde(default)]
    pub weapons: Vec<WeaponConfig>,
    #[serde(default)]
    pub armors: Vec<ArmorConfig>,
    #[serde(default)]
    pub utilities: Vec<UtilityConfig>,
    #[serde(default)]
    pub consumables: Vec<ConsumableConfig>,
    #[serde(default)]
    pub upgraders: Vec<UpgraderConfig>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    pub properties: WeaponProperties,
}

/// 武器属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponProperties {
    pub damage: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<i32>,
    pub votes: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aoe_damage: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bleed_damage: Option<i32>,
}

/// 防具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorConfig {
    pub internal_name: String,
    pub display_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    pub properties: ArmorProperties,
}

/// 防具属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorProperties {
    pub defense: i32,
    pub votes: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<i32>,
}

/// 工具 / 陷阱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    pub properties: UtilityProperties,
}

/// 消耗品配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    pub properties: ConsumableProperties,
}

/// 消耗品属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableProperties {
    pub effect_type: String,
    pub effect_value: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cure_bleed: Option<i32>,
}

/// 升级器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgraderConfig {
    pub internal_name: String,
    pub display_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
}

/// 工具 / 陷阱属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityProperties {
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub votes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses_night: Option<i32>,
}

/// 升级配方
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeRecipe {
    pub ingredients: Vec<String>,
    pub result: String,
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
                .get("items_config")
                .unwrap_or(&serde_json::json!({}))
                .clone(),
        )
        .map_err(|e| format!("Failed to parse items config: {}", e))?;

        // 解析队友行为配置
        let teammate_behavior = TeammateBehavior {
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
        for weapon in &self.items_config.items.weapons {
            if weapon.display_names.iter().any(|name| name == item_name) {
                return Ok(Item::new(
                    item_name.to_string(),
                    Some(weapon.internal_name.clone()),
                    weapon.rarity.clone(),
                    ItemType::Weapon(weapon.properties.clone()),
                ));
            }
        }

        // 2. 搜索防具
        for armor in &self.items_config.items.armors {
            if armor.display_names.iter().any(|name| name == item_name) {
                return Ok(Item::new(
                    item_name.to_string(),
                    Some(armor.internal_name.clone()),
                    armor.rarity.clone(),
                    ItemType::Armor(armor.properties.clone()),
                ));
            }
        }

        // 3. 搜索消耗品
        for consumable in &self.items_config.items.consumables {
            if consumable.name == item_name {
                return Ok(Item::new(
                    consumable.name.clone(),
                    consumable.internal_name.clone(),
                    consumable.rarity.clone(),
                    ItemType::Consumable(consumable.properties.clone()),
                ));
            }
        }

        // 4. 搜索工具/陷阱
        for utility in &self.items_config.items.utilities {
            if utility.name == item_name {
                return Ok(Item::new(
                    utility.name.clone(),
                    utility.internal_name.clone(),
                    utility.rarity.clone(),
                    ItemType::Utility(utility.properties.clone()),
                ));
            }
        }

        // 5. 搜索升级器
        for upgrader in &self.items_config.items.upgraders {
            if upgrader.display_names.iter().any(|name| name == item_name) {
                return Ok(Item::new(
                    item_name.to_string(),
                    Some(upgrader.internal_name.clone()),
                    upgrader.rarity.clone(),
                    ItemType::Upgrader,
                ));
            }
        }

        Err(format!("未在规则JSON中找到物品: {}", item_name))
    }

    /// 查找指定内部名称的武器配置
    pub fn find_weapon_config_by_internal_name(
        &self,
        internal_name: &str,
    ) -> Option<&WeaponConfig> {
        self.items_config
            .items
            .weapons
            .iter()
            .find(|config| config.internal_name == internal_name)
    }

    /// 查找指定内部名称的防具配置
    pub fn find_armor_config_by_internal_name(&self, internal_name: &str) -> Option<&ArmorConfig> {
        self.items_config
            .items
            .armors
            .iter()
            .find(|config| config.internal_name == internal_name)
    }
}
