//! Shared utilities for websocket actions.

use crate::game::game_rule_engine::UtilityProperties;

/// Formats numeric deltas with a leading sign when non-negative.
pub fn format_delta(value: i32) -> String {
    if value >= 0 {
        format!("+{}", value)
    } else {
        value.to_string()
    }
}

/// Use decrement mode for utility items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseMode {
    /// Decrement total remaining uses.
    Total,
    /// Decrement nightly remaining uses.
    Night,
    #[allow(dead_code)]
    /// Decrement both total and nightly uses.
    Both,
}

/// Outcome when decrementing a utility's use counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseOutcome {
    pub remaining_total: Option<i32>,
    pub remaining_night: Option<i32>,
    pub total_exhausted: bool,
    pub night_exhausted: bool,
}

/// Decrements the appropriate usage counters for a utility item.
pub fn decrement_uses(
    properties: &mut UtilityProperties,
    mode: UseMode,
) -> Result<UseOutcome, String> {
    if matches!(mode, UseMode::Night | UseMode::Both) {
        if let Some(uses_night) = properties.uses_night {
            if uses_night <= 0 {
                return Err("该物品今晚使用次数已用尽".to_string());
            }
        }
    }

    if matches!(mode, UseMode::Total | UseMode::Both) {
        if let Some(uses) = properties.uses {
            if uses <= 0 {
                return Err("该物品使用次数已用尽".to_string());
            }
        }
    }

    if matches!(mode, UseMode::Night | UseMode::Both) {
        if let Some(uses_night) = properties.uses_night {
            properties.uses_night = Some(uses_night - 1);
        }
    }

    if matches!(mode, UseMode::Total | UseMode::Both) {
        if let Some(uses) = properties.uses {
            properties.uses = Some(uses - 1);
        }
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
