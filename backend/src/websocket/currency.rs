//! Currency remains JSON-compatible with old saves, with exact half-unit arithmetic.
use serde::{Deserialize, Deserializer, Serializer};

// Twice this value is JavaScript's largest safe integer.
pub const MAX_COINS: f64 = 4_503_599_627_370_495.5;
pub const MAX_SELL_PRICE: f64 = 9999.0;

pub fn validate_balance(value: f64) -> Result<f64, String> {
    if !value.is_finite() || !(0.0..=MAX_COINS).contains(&value) || (value * 2.0).fract() != 0.0 {
        return Err(format!("货币必须为 0 到 {MAX_COINS} 范围内的 0.5 的倍数"));
    }
    Ok(value)
}

pub fn checked_change(balance: f64, delta: f64) -> Result<f64, String> {
    validate_balance(balance)?;
    if !delta.is_finite()
        || delta.abs() > MAX_COINS
        || (delta * 2.0).fract() != 0.0
        || delta < -balance
        || delta > MAX_COINS - balance
    {
        return Err("货币变更无效、余额不足或超过上限".to_string());
    }
    validate_balance(balance + delta)
}

pub fn valid_sell_price(price: f64) -> bool {
    price.is_finite() && (0.5..=MAX_SELL_PRICE).contains(&price) && (price * 2.0).fract() == 0.0
}

pub fn deserialize_balance<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    validate_balance(f64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
}

pub fn serialize_balance<S: Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    validate_balance(*value).map_err(serde::ser::Error::custom)?;
    serializer.serialize_f64(*value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_half_unit_arithmetic_at_boundaries() {
        assert_eq!(checked_change(MAX_COINS - 0.5, 0.5).unwrap(), MAX_COINS);
        assert_eq!(checked_change(MAX_COINS, -0.5).unwrap(), MAX_COINS - 0.5);
        assert_eq!(checked_change(0.5, -0.5).unwrap(), 0.0);
        for (balance, delta) in [
            (MAX_COINS, 0.5),
            (0.0, -0.5),
            (1e308, 1.0),
            (0.0, 1e308),
            (0.0, f64::NAN),
        ] {
            assert!(checked_change(balance, delta).is_err());
        }
    }
}
