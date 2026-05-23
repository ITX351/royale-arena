//! Shared utilities for websocket actions.

use crate::game::game_rule_engine::{GameRuleEngine, Item, ItemType, UtilityProperties};

/// Formats numeric deltas with a leading sign when non-negative.
pub fn format_delta(value: i32) -> String {
    if value >= 0 {
        format!("+{}", value)
    } else {
        value.to_string()
    }
}

/// Outcome when decrementing a utility's use counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseOutcome {
    pub remaining_total: Option<i32>,
    pub remaining_night: Option<i32>,
    pub total_exhausted: bool,
    pub night_exhausted: bool,
}

/// Decrements the usage counters for a utility item, requiring positive counts when present.
pub fn decrement_uses(properties: &mut UtilityProperties) -> Result<UseOutcome, String> {
    if let Some(uses_night) = properties.uses_night {
        if uses_night <= 0 {
            return Err("该物品今晚使用次数已用尽".to_string());
        }
    }

    if let Some(uses) = properties.uses {
        if uses <= 0 {
            return Err("该物品使用次数已用尽".to_string());
        }
    }

    if let Some(uses_night) = properties.uses_night {
        properties.uses_night = Some(uses_night - 1);
    }

    if let Some(uses) = properties.uses {
        properties.uses = Some(uses - 1);
    }

    let remaining_total = properties.uses;
    let remaining_night = properties.uses_night;

    Ok(UseOutcome {
        remaining_total,
        remaining_night,
        total_exhausted: remaining_total.map(|value| value <= 0).unwrap_or(false),
        night_exhausted: remaining_night.map(|value| value <= 0).unwrap_or(false),
    })
}

/// Formats a suffix describing remaining uses for logging.
pub fn format_use_remaining_suffix(use_outcome: &UseOutcome) -> Option<String> {
    let mut segments: Vec<String> = Vec::new();

    if let Some(remaining) = use_outcome.remaining_total {
        if remaining > 0 {
            segments.push(format!("累计剩余 {} 次", remaining));
        } else {
            segments.push("道具已耗尽".to_string());
        }
    }

    if let Some(remaining) = use_outcome.remaining_night {
        segments.push(format!("本晚剩余 {} 次", remaining.max(0)));
    }

    if segments.is_empty() {
        None
    } else {
        Some(format!("（{}）", segments.join("，")))
    }
}

/// Restores nightly-use counters for utility items based on their template defaults.
pub fn restore_item_nightly_uses(item: &mut Item, rule_engine: &GameRuleEngine) {
    if let ItemType::Utility(ref mut properties) = item.item_type {
        if properties.uses_night.is_some() {
            if let Ok(template) = rule_engine.create_item_from_name(&item.name) {
                if let ItemType::Utility(default_properties) = template.item_type {
                    if let Some(default) = default_properties.uses_night {
                        properties.uses_night = Some(default);
                    }
                }
            }
        }
    }
}
