use crate::*;

use super::{eval_support::binary, support::scalar};

fn integer(value: i64) -> TemporalValue {
    TemporalValue::Integer { value }
}

fn length(value: f64, unit: LengthUnit) -> TemporalValue {
    TemporalValue::Length {
        value: Length { value, unit },
    }
}

#[test]
fn arithmetic_failures_are_typed_and_bounded() {
    use TemporalBinaryOperation as Op;
    let error = binary(Op::Divide, scalar(1.0), scalar(0.0), TemporalType::Scalar).unwrap_err();
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
    let error = binary(
        Op::Add,
        integer(i64::try_from(MAX_SAFE_INTEGER).unwrap()),
        integer(1),
        TemporalType::Integer,
    )
    .unwrap_err();
    assert!(error.message().contains("exact numeric range"));
    let error = binary(
        Op::Add,
        length(1.0, LengthUnit::Pixels),
        length(1.0, LengthUnit::Percent),
        TemporalType::Length,
    )
    .unwrap_err();
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
}
