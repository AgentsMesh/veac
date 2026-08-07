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
fn integer_and_scalar_arithmetic_is_checked() {
    use TemporalBinaryOperation as Op;
    assert_eq!(
        binary(Op::Add, integer(6), integer(2), TemporalType::Integer).unwrap(),
        integer(8)
    );
    assert_eq!(
        binary(Op::Subtract, integer(6), integer(2), TemporalType::Integer).unwrap(),
        integer(4)
    );
    assert_eq!(
        binary(Op::Multiply, integer(6), integer(2), TemporalType::Integer).unwrap(),
        integer(12)
    );
    assert_eq!(
        binary(Op::Divide, integer(7), integer(2), TemporalType::Integer).unwrap(),
        integer(3)
    );
    assert_eq!(
        binary(Op::Minimum, integer(6), integer(2), TemporalType::Integer).unwrap(),
        integer(2)
    );
    assert_eq!(
        binary(Op::Maximum, integer(6), integer(2), TemporalType::Integer).unwrap(),
        integer(6)
    );
    assert_eq!(
        binary(Op::Add, scalar(1.5), scalar(2.0), TemporalType::Scalar).unwrap(),
        scalar(3.5)
    );
    assert_eq!(
        binary(Op::Subtract, scalar(3.5), scalar(2.0), TemporalType::Scalar).unwrap(),
        scalar(1.5)
    );
    assert_eq!(
        binary(Op::Multiply, scalar(1.5), scalar(2.0), TemporalType::Scalar).unwrap(),
        scalar(3.0)
    );
    assert_eq!(
        binary(Op::Divide, scalar(3.0), scalar(2.0), TemporalType::Scalar).unwrap(),
        scalar(1.5)
    );
}

#[test]
fn typed_arithmetic_preserves_time_units_and_vector_shape() {
    use TemporalBinaryOperation as Op;
    let time = |value| TemporalValue::Time {
        value: RationalTime::new(value, 30).unwrap(),
    };
    assert_eq!(
        binary(Op::Add, time(3), time(2), TemporalType::Time).unwrap(),
        time(5)
    );
    assert_eq!(
        binary(Op::Subtract, time(3), time(2), TemporalType::Time).unwrap(),
        time(1)
    );
    assert_eq!(
        binary(
            Op::Add,
            length(3.0, LengthUnit::Pixels),
            length(2.0, LengthUnit::Pixels),
            TemporalType::Length
        )
        .unwrap(),
        length(5.0, LengthUnit::Pixels)
    );
    assert_eq!(
        binary(
            Op::Multiply,
            length(3.0, LengthUnit::Pixels),
            scalar(2.0),
            TemporalType::Length
        )
        .unwrap(),
        length(6.0, LengthUnit::Pixels)
    );
    assert_eq!(
        binary(
            Op::Multiply,
            scalar(2.0),
            length(3.0, LengthUnit::Pixels),
            TemporalType::Length
        )
        .unwrap(),
        length(6.0, LengthUnit::Pixels)
    );
    assert_eq!(
        binary(
            Op::Divide,
            length(6.0, LengthUnit::Pixels),
            scalar(2.0),
            TemporalType::Length
        )
        .unwrap(),
        length(3.0, LengthUnit::Pixels)
    );
    let angle = |degrees| TemporalValue::Angle { degrees };
    assert_eq!(
        binary(Op::Add, angle(20.0), angle(10.0), TemporalType::Angle).unwrap(),
        angle(30.0)
    );
    assert_eq!(
        binary(Op::Multiply, angle(20.0), scalar(2.0), TemporalType::Angle).unwrap(),
        angle(40.0)
    );
    assert_eq!(
        binary(Op::Divide, angle(20.0), scalar(2.0), TemporalType::Angle).unwrap(),
        angle(10.0)
    );
}

#[test]
fn vector_arithmetic_supports_component_and_scalar_forms() {
    use TemporalBinaryOperation as Op;
    let vector = |x, y| TemporalValue::Vec2 {
        value: Vec2 { x, y },
    };
    assert_eq!(
        binary(
            Op::Add,
            vector(1.0, 2.0),
            vector(3.0, 4.0),
            TemporalType::Vec2
        )
        .unwrap(),
        vector(4.0, 6.0)
    );
    assert_eq!(
        binary(
            Op::Subtract,
            vector(3.0, 4.0),
            vector(1.0, 2.0),
            TemporalType::Vec2
        )
        .unwrap(),
        vector(2.0, 2.0)
    );
    assert_eq!(
        binary(
            Op::Multiply,
            vector(1.0, 2.0),
            scalar(3.0),
            TemporalType::Vec2
        )
        .unwrap(),
        vector(3.0, 6.0)
    );
    assert_eq!(
        binary(
            Op::Multiply,
            scalar(3.0),
            vector(1.0, 2.0),
            TemporalType::Vec2
        )
        .unwrap(),
        vector(3.0, 6.0)
    );
    assert_eq!(
        binary(
            Op::Divide,
            vector(3.0, 6.0),
            scalar(3.0),
            TemporalType::Vec2
        )
        .unwrap(),
        vector(1.0, 2.0)
    );
}
