use crate::*;

use super::support::{literal, program, scalar};

fn time() -> TemporalValue {
    TemporalValue::Time {
        value: RationalTime::new(4_800, 600).unwrap(),
    }
}

fn valid_binary(
    operation: TemporalBinaryOperation,
    left: TemporalValue,
    right: TemporalValue,
    result_type: TemporalType,
) {
    let nodes = vec![
        literal(0, left),
        literal(1, right),
        TemporalNode {
            id: TemporalNodeId::new(2),
            value_type: result_type,
            kind: TemporalNodeKind::Binary {
                operation,
                left: TemporalNodeId::new(0),
                right: TemporalNodeId::new(1),
            },
            provenance_id: None,
        },
    ];
    validate_temporal_program(&program(Vec::new(), nodes, 2, result_type)).unwrap();
}

#[test]
fn time_and_scalar_arithmetic_is_a_closed_typed_contract() {
    use TemporalBinaryOperation as Op;
    valid_binary(Op::Multiply, time(), scalar(0.5), TemporalType::Time);
    valid_binary(Op::Multiply, scalar(0.5), time(), TemporalType::Time);
    valid_binary(Op::Divide, time(), scalar(2.0), TemporalType::Time);
}

#[test]
fn dimensional_division_has_a_scalar_result_contract() {
    use TemporalBinaryOperation as Op;
    valid_binary(Op::Divide, time(), time(), TemporalType::Scalar);
    valid_binary(
        Op::Divide,
        TemporalValue::Length {
            value: Length {
                value: 2.0,
                unit: LengthUnit::Pixels,
            },
        },
        TemporalValue::Length {
            value: Length {
                value: 1.0,
                unit: LengthUnit::Pixels,
            },
        },
        TemporalType::Scalar,
    );
    valid_binary(
        Op::Divide,
        TemporalValue::Angle { degrees: 90.0 },
        TemporalValue::Angle { degrees: 45.0 },
        TemporalType::Scalar,
    );
}
