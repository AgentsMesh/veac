use crate::*;

use super::support::{literal, program, scalar};

fn evaluate(
    nodes: Vec<TemporalNode>,
    result: u32,
    result_type: TemporalType,
    max_steps: usize,
    max_value_bytes: usize,
) -> Result<TemporalValue, TemporalEvaluationError> {
    evaluate_temporal_program(
        &program(Vec::new(), nodes, result, result_type),
        &[],
        TemporalEvaluationLimits {
            max_steps,
            max_value_bytes,
        },
    )
}

fn binary(id: u32, operation: TemporalBinaryOperation, left: u32, right: u32) -> TemporalNode {
    TemporalNode {
        id: TemporalNodeId::new(id),
        value_type: TemporalType::Scalar,
        kind: TemporalNodeKind::Binary {
            operation,
            left: TemporalNodeId::new(left),
            right: TemporalNodeId::new(right),
        },
        provenance_id: None,
    }
}

#[test]
fn only_result_reachable_nodes_consume_budget_or_execute() {
    let nodes = vec![
        literal(0, scalar(2.0)),
        literal(1, scalar(0.0)),
        binary(2, TemporalBinaryOperation::Divide, 0, 1),
    ];
    assert_eq!(
        evaluate(nodes, 0, TemporalType::Scalar, 1, 8).unwrap(),
        scalar(2.0)
    );
}

#[test]
fn shared_dependencies_are_memoized_once() {
    let nodes = vec![
        literal(0, scalar(2.0)),
        binary(1, TemporalBinaryOperation::Add, 0, 0),
    ];
    assert_eq!(
        evaluate(nodes, 1, TemporalType::Scalar, 2, 16).unwrap(),
        scalar(4.0)
    );
}

#[test]
fn select_evaluates_only_the_selected_branch() {
    let nodes = vec![
        literal(0, TemporalValue::Boolean { value: true }),
        literal(1, scalar(10.0)),
        literal(2, scalar(0.0)),
        binary(3, TemporalBinaryOperation::Divide, 1, 2),
        TemporalNode {
            id: TemporalNodeId::new(4),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::Select {
                condition: TemporalNodeId::new(0),
                when_true: TemporalNodeId::new(1),
                when_false: TemporalNodeId::new(3),
            },
            provenance_id: None,
        },
    ];
    assert_eq!(
        evaluate(nodes, 4, TemporalType::Scalar, 3, 17).unwrap(),
        scalar(10.0)
    );
}
