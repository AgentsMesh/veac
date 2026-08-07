use crate::*;

use super::{eval_support::unary, support::scalar};

#[test]
fn boolean_and_numeric_unary_operations_are_deterministic() {
    assert_eq!(
        unary(
            TemporalUnaryOperation::Not,
            TemporalValue::Boolean { value: true },
            TemporalType::Boolean
        )
        .unwrap(),
        TemporalValue::Boolean { value: false }
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Negate,
            TemporalValue::Integer { value: 7 },
            TemporalType::Integer
        )
        .unwrap(),
        TemporalValue::Integer { value: -7 }
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Absolute,
            TemporalValue::Integer { value: -7 },
            TemporalType::Integer
        )
        .unwrap(),
        TemporalValue::Integer { value: 7 }
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Negate,
            scalar(2.0),
            TemporalType::Scalar
        )
        .unwrap(),
        scalar(-2.0)
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Absolute,
            scalar(-2.0),
            TemporalType::Scalar
        )
        .unwrap(),
        scalar(2.0)
    );
}

#[test]
fn typed_time_length_angle_and_vector_unary_operations_preserve_units() {
    let time = TemporalValue::Time {
        value: RationalTime::new(2, 30).unwrap(),
    };
    assert_eq!(
        unary(TemporalUnaryOperation::Negate, time, TemporalType::Time).unwrap(),
        TemporalValue::Time {
            value: RationalTime::new(-2, 30).unwrap()
        }
    );
    let length = TemporalValue::Length {
        value: Length {
            value: -3.0,
            unit: LengthUnit::Pixels,
        },
    };
    assert_eq!(
        unary(
            TemporalUnaryOperation::Absolute,
            length,
            TemporalType::Length
        )
        .unwrap(),
        TemporalValue::Length {
            value: Length {
                value: 3.0,
                unit: LengthUnit::Pixels
            }
        }
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Negate,
            TemporalValue::Angle { degrees: 45.0 },
            TemporalType::Angle
        )
        .unwrap(),
        TemporalValue::Angle { degrees: -45.0 }
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Absolute,
            TemporalValue::Vec2 {
                value: Vec2 { x: -1.0, y: 2.0 }
            },
            TemporalType::Vec2
        )
        .unwrap(),
        TemporalValue::Vec2 {
            value: Vec2 { x: 1.0, y: 2.0 }
        }
    );
}

#[test]
fn scalar_math_and_degree_trigonometry_have_reference_semantics() {
    assert_eq!(
        unary(
            TemporalUnaryOperation::Floor,
            scalar(1.8),
            TemporalType::Scalar
        )
        .unwrap(),
        scalar(1.0)
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Ceil,
            scalar(1.2),
            TemporalType::Scalar
        )
        .unwrap(),
        scalar(2.0)
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::Round,
            scalar(1.6),
            TemporalType::Scalar
        )
        .unwrap(),
        scalar(2.0)
    );
    assert_eq!(
        unary(
            TemporalUnaryOperation::SquareRoot,
            scalar(9.0),
            TemporalType::Scalar
        )
        .unwrap(),
        scalar(3.0)
    );
    let exponential = unary(
        TemporalUnaryOperation::Exponential,
        scalar(1.0),
        TemporalType::Scalar,
    )
    .unwrap();
    assert!(
        matches!(exponential, TemporalValue::Scalar { value } if (value - std::f64::consts::E).abs() < 1e-12)
    );
    let logarithm = unary(
        TemporalUnaryOperation::NaturalLog,
        scalar(std::f64::consts::E),
        TemporalType::Scalar,
    )
    .unwrap();
    assert!(matches!(logarithm, TemporalValue::Scalar { value } if (value - 1.0).abs() < 1e-12));
    let sine = unary(
        TemporalUnaryOperation::Sine,
        TemporalValue::Angle { degrees: 90.0 },
        TemporalType::Scalar,
    )
    .unwrap();
    assert!(matches!(sine, TemporalValue::Scalar { value } if (value - 1.0).abs() < 1e-12));
    assert_eq!(
        unary(
            TemporalUnaryOperation::Cosine,
            TemporalValue::Angle { degrees: 180.0 },
            TemporalType::Scalar
        )
        .unwrap(),
        scalar(-1.0)
    );
}

#[test]
fn non_finite_unary_results_are_errors_not_process_failures() {
    let error = unary(
        TemporalUnaryOperation::SquareRoot,
        scalar(-1.0),
        TemporalType::Scalar,
    )
    .unwrap_err();
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
    let error = unary(
        TemporalUnaryOperation::Exponential,
        scalar(1000.0),
        TemporalType::Scalar,
    )
    .unwrap_err();
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
}
