use super::exact;
use crate::program::expression::{PrimitiveType, Value};

pub(super) fn unit_value(raw: &str, kind: PrimitiveType) -> Option<Value> {
    let exact = match kind {
        PrimitiveType::Time => exact::unit(raw, "s", (1, 1))
            .or_else(|| exact::unit(raw, "ms", (1, 1_000)))
            .or_else(|| exact::unit(raw, "us", (1, 1_000_000))),
        PrimitiveType::Length => exact::unit(raw, "px", (1, 1)),
        PrimitiveType::Angle => exact::unit(raw, "deg", (1, 1)),
        _ => None,
    }?;
    match kind {
        PrimitiveType::Time => Some(Value::Time(exact)),
        PrimitiveType::Length => Some(Value::Length(exact)),
        PrimitiveType::Angle => Some(Value::Angle(exact)),
        _ => None,
    }
}

pub(super) fn valid_color(value: &str) -> bool {
    matches!(value.len(), 7 | 9)
        && value.starts_with('#')
        && value[1..].bytes().all(|value| value.is_ascii_hexdigit())
}
