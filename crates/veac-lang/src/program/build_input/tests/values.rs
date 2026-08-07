use crate::program::expression::{ExactNumber, Value};

use super::*;

#[test]
fn exact_decimal_and_unit_parsing_rejects_noncanonical_or_overflowing_values() {
    assert_eq!(exact::decimal("0"), ExactNumber::new(0, 1));
    assert_eq!(exact::decimal("-1.25"), ExactNumber::new(-5, 4));
    for invalid in [
        "",
        " 1",
        "+1",
        ".5",
        "1.",
        "1e3",
        "9999999999999999999999999999999999999999",
    ] {
        assert_eq!(exact::decimal(invalid), None, "{invalid}");
    }
    assert_eq!(
        exact::unit("1250ms", "ms", (1, 1_000)),
        ExactNumber::new(5, 4)
    );
    assert_eq!(exact::unit("1s", "ms", (1, 1_000)), None);
    assert_eq!(
        exact::unit("1ms", "ms", (u64::MAX, u64::MAX)),
        ExactNumber::new(1, 1)
    );
    assert_eq!(
        exact::unit(
            "170141183460469231731687303715884105727ms",
            "ms",
            (u64::MAX, 1),
        ),
        None
    );
}

#[test]
fn runtime_values_reject_invalid_literals_without_partial_coercion() {
    for value in [
        BuildInputManifestValue::Scalar {
            value: "NaN".to_owned(),
        },
        BuildInputManifestValue::Time {
            value: "1px".to_owned(),
        },
        BuildInputManifestValue::Length {
            value: "1deg".to_owned(),
        },
        BuildInputManifestValue::Angle {
            value: "1s".to_owned(),
        },
        BuildInputManifestValue::Color {
            value: "#xyzxyz".to_owned(),
        },
        BuildInputManifestValue::Text {
            value: "x".repeat(crate::program::expression::MAX_TEXT_VALUE_BYTES + 1),
        },
    ] {
        let error = value.runtime_value().unwrap_err();
        assert_eq!(error.code(), "PROGRAM_INPUT_VALUE");
        assert!(error.message().contains("invalid"));
    }
}

#[test]
fn runtime_values_preserve_exact_units_and_normalize_colors() {
    assert!(matches!(
        BuildInputManifestValue::Bool { value: true }.runtime_value(),
        Ok(Value::Bool(true))
    ));
    assert!(matches!(
        BuildInputManifestValue::Integer { value: -7 }.runtime_value(),
        Ok(Value::Integer(-7))
    ));
    assert_eq!(
        BuildInputManifestValue::Time {
            value: "1250ms".to_owned(),
        }
        .runtime_value(),
        Ok(Value::Time(ExactNumber::new(5, 4).unwrap()))
    );
    assert_eq!(
        BuildInputManifestValue::Color {
            value: "#AABBCCDD".to_owned(),
        }
        .runtime_value(),
        Ok(Value::Color("#aabbccdd".into()))
    );
}
