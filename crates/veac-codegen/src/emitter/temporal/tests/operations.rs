use veac_plan::canonical::*;

use super::support::{compile, ffmpeg, node, numeric_expression, plan, program, scalar};

#[test]
fn every_unary_operation_compiles_to_an_ffmpeg_expression() {
    use TemporalUnaryOperation as Op;
    for (operation, input) in [
        (Op::Not, boolean(true)),
        (Op::Negate, scalar(1.25)),
        (Op::Absolute, scalar(-1.25)),
        (Op::Floor, scalar(1.75)),
        (Op::Ceil, scalar(1.25)),
        (Op::Round, scalar(-1.5)),
        (Op::SquareRoot, scalar(4.0)),
        (Op::Sine, scalar(30.0)),
        (Op::Cosine, scalar(60.0)),
        (Op::Exponential, scalar(1.0)),
        (Op::NaturalLog, scalar(2.0)),
    ] {
        let input_type = input.value_type();
        let result_type = if operation == Op::Not {
            TemporalType::Boolean
        } else {
            TemporalType::Scalar
        };
        let nodes = vec![
            node(0, input_type, TemporalNodeKind::Literal { value: input }),
            node(
                1,
                result_type,
                TemporalNodeKind::Unary {
                    operation,
                    operand: TemporalNodeId::new(0),
                },
            ),
        ];
        assert_expression_runs(nodes, result_type);
    }
}

#[test]
fn every_binary_and_compare_operation_compiles_to_ffmpeg() {
    use TemporalBinaryOperation as Binary;
    for operation in [
        Binary::Add,
        Binary::Subtract,
        Binary::Multiply,
        Binary::Divide,
        Binary::Minimum,
        Binary::Maximum,
    ] {
        assert_expression_runs(
            binary_nodes(operation, scalar(6.0), scalar(2.0)),
            TemporalType::Scalar,
        );
    }
    use TemporalCompareOperation as Compare;
    for operation in [
        Compare::Equal,
        Compare::NotEqual,
        Compare::Less,
        Compare::LessOrEqual,
        Compare::Greater,
        Compare::GreaterOrEqual,
    ] {
        let mut nodes = literal_pair(scalar(6.0), scalar(2.0));
        nodes.push(node(
            2,
            TemporalType::Boolean,
            TemporalNodeKind::Compare {
                operation,
                left: TemporalNodeId::new(0),
                right: TemporalNodeId::new(1),
            },
        ));
        assert_expression_runs(nodes, TemporalType::Boolean);
    }
}

#[test]
fn integer_division_uses_an_ffmpeg_supported_truncation_expression() {
    let nodes = binary_nodes(
        TemporalBinaryOperation::Divide,
        TemporalValue::Integer { value: -5 },
        TemporalValue::Integer { value: 2 },
    );
    let expression = expression(nodes, TemporalType::Integer);
    assert!(expression.contains("trunc("), "{expression}");
    assert_eq!(ffmpeg(&expression), -2.0);
}

fn binary_nodes(
    operation: TemporalBinaryOperation,
    left: TemporalValue,
    right: TemporalValue,
) -> Vec<TemporalNode> {
    let result_type = match (&left, operation) {
        (TemporalValue::Integer { .. }, _) => TemporalType::Integer,
        _ => TemporalType::Scalar,
    };
    let mut nodes = literal_pair(left, right);
    nodes.push(node(
        2,
        result_type,
        TemporalNodeKind::Binary {
            operation,
            left: TemporalNodeId::new(0),
            right: TemporalNodeId::new(1),
        },
    ));
    nodes
}

fn literal_pair(left: TemporalValue, right: TemporalValue) -> Vec<TemporalNode> {
    vec![
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
    ]
}

fn assert_expression_runs(nodes: Vec<TemporalNode>, result_type: TemporalType) {
    assert!(ffmpeg(&expression(nodes, result_type)).is_finite());
}

fn expression(nodes: Vec<TemporalNode>, result_type: TemporalType) -> String {
    let result = nodes.len() as u32 - 1;
    let (plan, binding) = plan(
        program(Vec::new(), nodes, result, result_type),
        Vec::new(),
        Vec::new(),
    );
    numeric_expression(&compile(&plan, &binding, "t").unwrap()).to_owned()
}

fn boolean(value: bool) -> TemporalValue {
    TemporalValue::Boolean { value }
}
