//! Shared utilities for websocket actions.

/// Formats numeric deltas with a leading sign when non-negative.
pub fn format_delta(value: i32) -> String {
    if value >= 0 {
        format!("+{}", value)
    } else {
        value.to_string()
    }
}
