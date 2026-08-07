use crate::*;

use super::{
    eval_support::{binary, compare_result, evaluate},
    support::{literal, program, scalar},
};

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
fn cross_timescale_arithmetic_is_exact_and_reduced() {
    use TemporalBinaryOperation as Op;
    assert_eq!(
        binary(Op::Add, time(1, 30), time(1, 24), TemporalType::Time).unwrap(),
        time(3, 40)
    );
    assert_eq!(
        binary(Op::Subtract, time(1, 24), time(1, 30), TemporalType::Time).unwrap(),
        time(1, 120)
    );
    assert_eq!(
        binary(Op::Minimum, time(1, 24), time(1, 30), TemporalType::Time).unwrap(),
        time(1, 30)
    );
    assert_eq!(
        binary(Op::Maximum, time(1, 24), time(1, 30), TemporalType::Time).unwrap(),
        time(1, 24)
    );
}

#[test]
fn unrepresentable_exact_time_results_are_value_errors() {
    let error = binary(
        TemporalBinaryOperation::Add,
        time(1, u32::MAX),
        time(1, u32::MAX - 1),
        TemporalType::Time,
    )
    .unwrap_err();
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");

    let error = binary(
        TemporalBinaryOperation::Add,
        time(i64::try_from(MAX_SAFE_INTEGER).unwrap(), 30),
        time(1, 30),
        TemporalType::Time,
    )
    .unwrap_err();
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
}

#[test]
fn time_curve_progress_uses_exact_integer_differences() {
    let maximum = i64::try_from(MAX_SAFE_INTEGER).unwrap();
    let keys = vec![
        TemporalCurveKey {
            position: TemporalCurvePosition::Time {
                value: RationalTime::new(maximum - 3, 3).unwrap(),
            },
            value: scalar(0.0),
            interpolation: Interpolation::Linear,
        },
        TemporalCurveKey {
            position: TemporalCurvePosition::Time {
                value: RationalTime::new(maximum, 3).unwrap(),
            },
            value: scalar(9.0),
            interpolation: Interpolation::Hold,
        },
    ];
    let nodes = vec![
        literal(0, time(maximum - 2, 3)),
        TemporalNode {
            id: TemporalNodeId::new(1),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::CurveSample {
                input: TemporalNodeId::new(0),
                keys,
            },
            provenance_id: None,
        },
    ];
    assert_eq!(
        evaluate(&program(Vec::new(), nodes, 1, TemporalType::Scalar), &[]),
        scalar(3.0)
    );
}

#[test]
fn relative_lengths_convert_to_one_canonical_unit() {
    use TemporalBinaryOperation as Op;
    let normalized = |value| length(value, LengthUnit::Normalized);
    let percent = |value| length(value, LengthUnit::Percent);
    assert_eq!(
        binary(
            Op::Add,
            normalized(1.0),
            percent(50.0),
            TemporalType::Length
        )
        .unwrap(),
        normalized(1.5)
    );
    assert_eq!(
        binary(
            Op::Subtract,
            percent(50.0),
            normalized(0.25),
            TemporalType::Length
        )
        .unwrap(),
        normalized(0.25)
    );
    assert_eq!(
        binary(
            Op::Maximum,
            normalized(0.5),
            percent(75.0),
            TemporalType::Length
        )
        .unwrap(),
        normalized(0.75)
    );
    assert_eq!(
        compare_result(
            TemporalCompareOperation::Equal,
            normalized(1.0),
            percent(100.0)
        )
        .unwrap(),
        TemporalValue::Boolean { value: true }
    );
    assert_eq!(
        compare_result(
            TemporalCompareOperation::Less,
            normalized(0.5),
            percent(75.0)
        )
        .unwrap(),
        TemporalValue::Boolean { value: true }
    );
}

#[test]
fn pixel_relative_operations_report_stable_value_errors() {
    let pixels = length(10.0, LengthUnit::Pixels);
    let percent = length(10.0, LengthUnit::Percent);
    for error in [
        binary(
            TemporalBinaryOperation::Minimum,
            pixels.clone(),
            percent.clone(),
            TemporalType::Length,
        )
        .unwrap_err(),
        compare_result(TemporalCompareOperation::Equal, pixels, percent).unwrap_err(),
    ] {
        assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
    }
}
