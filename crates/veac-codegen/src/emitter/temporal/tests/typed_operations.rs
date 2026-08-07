use veac_plan::canonical::*;

use super::support::{compile, node, plan, program};
use super::CompiledValue;

#[test]
fn unary_mapping_covers_every_supported_typed_family() {
    let cases = [
        value_case(integer(-2), TemporalUnaryOperation::Negate),
        value_case(time(-300), TemporalUnaryOperation::Absolute),
        value_case(length(-3.0), TemporalUnaryOperation::Negate),
        value_case(angle(-30.0), TemporalUnaryOperation::Absolute),
        value_case(vector(-2.0), TemporalUnaryOperation::Negate),
    ];
    for (value, operation) in cases {
        let result_type = value.value_type();
        let nodes = vec![
            node(0, result_type, TemporalNodeKind::Literal { value }),
            node(
                1,
                result_type,
                TemporalNodeKind::Unary {
                    operation,
                    operand: TemporalNodeId::new(0),
                },
            ),
        ];
        assert_variant(compile_nodes(nodes, result_type), result_type);
    }
}

#[test]
fn binary_mapping_covers_integer_and_dimensioned_arithmetic() {
    use TemporalBinaryOperation as Op;
    for operation in [
        Op::Add,
        Op::Subtract,
        Op::Multiply,
        Op::Divide,
        Op::Minimum,
        Op::Maximum,
    ] {
        assert_variant(
            binary(integer(7), integer(2), operation),
            TemporalType::Integer,
        );
    }
    for (left, right, operation, result) in [
        (time(600), time(300), Op::Add, TemporalType::Time),
        (time(600), time(300), Op::Divide, TemporalType::Scalar),
        (time(600), scalar(2.0), Op::Multiply, TemporalType::Time),
        (time(600), scalar(2.0), Op::Divide, TemporalType::Time),
        (scalar(2.0), time(600), Op::Multiply, TemporalType::Time),
        (angle(60.0), angle(30.0), Op::Add, TemporalType::Angle),
        (angle(60.0), angle(30.0), Op::Divide, TemporalType::Scalar),
        (angle(60.0), scalar(2.0), Op::Multiply, TemporalType::Angle),
        (length(12.0), length(3.0), Op::Add, TemporalType::Length),
        (length(12.0), length(3.0), Op::Divide, TemporalType::Scalar),
        (length(12.0), scalar(3.0), Op::Divide, TemporalType::Length),
        (
            scalar(3.0),
            length(12.0),
            Op::Multiply,
            TemporalType::Length,
        ),
        (vector(2.0), vector(3.0), Op::Add, TemporalType::Vec2),
        (vector(2.0), scalar(3.0), Op::Multiply, TemporalType::Vec2),
        (scalar(3.0), vector(2.0), Op::Multiply, TemporalType::Vec2),
    ] {
        assert_variant(binary(left, right, operation), result);
    }
}

fn binary(
    left: TemporalValue,
    right: TemporalValue,
    operation: TemporalBinaryOperation,
) -> CompiledValue {
    let result_type = binary_result(&left, &right, operation);
    let nodes = vec![
        node(
            0,
            left.value_type(),
            TemporalNodeKind::Literal { value: left },
        ),
        node(
            1,
            right.value_type(),
            TemporalNodeKind::Literal { value: right },
        ),
        node(
            2,
            result_type,
            TemporalNodeKind::Binary {
                operation,
                left: TemporalNodeId::new(0),
                right: TemporalNodeId::new(1),
            },
        ),
    ];
    compile_nodes(nodes, result_type)
}

fn binary_result(
    left: &TemporalValue,
    right: &TemporalValue,
    operation: TemporalBinaryOperation,
) -> TemporalType {
    if operation == TemporalBinaryOperation::Divide && left.value_type() == right.value_type() {
        return match left {
            TemporalValue::Time { .. }
            | TemporalValue::Length { .. }
            | TemporalValue::Angle { .. } => TemporalType::Scalar,
            _ => left.value_type(),
        };
    }
    if matches!(left, TemporalValue::Scalar { .. }) {
        right.value_type()
    } else {
        left.value_type()
    }
}

fn compile_nodes(nodes: Vec<TemporalNode>, result_type: TemporalType) -> CompiledValue {
    let result = nodes.len() as u32 - 1;
    let (plan, binding) = plan(
        program(Vec::new(), nodes, result, result_type),
        Vec::new(),
        Vec::new(),
    );
    compile(&plan, &binding, "t").unwrap()
}

fn assert_variant(value: CompiledValue, expected: TemporalType) {
    let actual = match value {
        CompiledValue::Integer(_) => TemporalType::Integer,
        CompiledValue::Scalar(_) => TemporalType::Scalar,
        CompiledValue::Time(_) => TemporalType::Time,
        CompiledValue::Length(_) => TemporalType::Length,
        CompiledValue::Angle(_) => TemporalType::Angle,
        CompiledValue::Vec2(_, _) => TemporalType::Vec2,
        _ => panic!("unexpected compiled value"),
    };
    assert_eq!(actual, expected);
}

fn value_case(
    value: TemporalValue,
    operation: TemporalUnaryOperation,
) -> (TemporalValue, TemporalUnaryOperation) {
    (value, operation)
}

fn integer(value: i64) -> TemporalValue {
    TemporalValue::Integer { value }
}
fn scalar(value: f64) -> TemporalValue {
    TemporalValue::Scalar { value }
}
fn time(value: i64) -> TemporalValue {
    TemporalValue::Time {
        value: RationalTime::new(value, 600).unwrap(),
    }
}
fn length(value: f64) -> TemporalValue {
    TemporalValue::Length {
        value: Length {
            value,
            unit: LengthUnit::Pixels,
        },
    }
}
fn angle(degrees: f64) -> TemporalValue {
    TemporalValue::Angle { degrees }
}
fn vector(value: f64) -> TemporalValue {
    TemporalValue::Vec2 {
        value: Vec2 { x: value, y: value },
    }
}
