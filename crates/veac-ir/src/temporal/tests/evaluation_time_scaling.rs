use crate::*;

use super::{eval_support::binary, support::scalar};

fn time(value: i64, timescale: u32) -> TemporalValue {
    TemporalValue::Time {
        value: RationalTime::new(value, timescale).unwrap(),
    }
}

fn length(value: f64, unit: LengthUnit) -> TemporalValue {
    TemporalValue::Length {
        value: Length { value, unit },
    }
}

#[test]
fn time_scaling_is_closed_for_multiply_and_divide() {
    use TemporalBinaryOperation as Op;
    let eight_seconds = time(4_800, 600);
    for (left, right) in [
        (scalar(0.5), eight_seconds.clone()),
        (eight_seconds.clone(), scalar(0.5)),
    ] {
        assert_eq!(
            binary(Op::Multiply, left, right, TemporalType::Time).unwrap(),
            time(2_400, 600)
        );
    }
    assert_eq!(
        binary(Op::Divide, eight_seconds, scalar(2.0), TemporalType::Time).unwrap(),
        time(2_400, 600)
    );
}

#[test]
fn dimensional_division_returns_a_scalar_ratio() {
    use TemporalBinaryOperation as Op;
    for (left, right) in [
        (time(3, 30), time(1, 20)),
        (
            length(50.0, LengthUnit::Percent),
            length(0.25, LengthUnit::Normalized),
        ),
        (
            TemporalValue::Angle { degrees: 90.0 },
            TemporalValue::Angle { degrees: 45.0 },
        ),
    ] {
        assert_eq!(
            binary(Op::Divide, left, right, TemporalType::Scalar).unwrap(),
            scalar(2.0)
        );
    }
}

#[test]
fn dimensional_division_rejects_zero_and_incompatible_units() {
    use TemporalBinaryOperation as Op;
    for (left, right) in [
        (time(1, 600), time(0, 600)),
        (
            length(1.0, LengthUnit::Pixels),
            length(0.0, LengthUnit::Pixels),
        ),
        (
            TemporalValue::Angle { degrees: 1.0 },
            TemporalValue::Angle { degrees: 0.0 },
        ),
        (
            length(1.0, LengthUnit::Pixels),
            length(1.0, LengthUnit::Percent),
        ),
    ] {
        let error = binary(Op::Divide, left, right, TemporalType::Scalar).unwrap_err();
        assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
    }
}

#[test]
fn time_scaling_rejects_fractional_ticks_and_zero_divisors() {
    use TemporalBinaryOperation as Op;
    for (left, right) in [(time(1, 600), scalar(0.5)), (time(600, 600), scalar(0.0))] {
        let operation = if matches!(right, TemporalValue::Scalar { value: 0.0 }) {
            Op::Divide
        } else {
            Op::Multiply
        };
        let error = binary(operation, left, right, TemporalType::Time).unwrap_err();
        assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
    }
}
