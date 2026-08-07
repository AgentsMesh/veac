use crate::*;

use super::support::{literal, program};

pub(super) fn evaluate(
    program: &TemporalProgram,
    inputs: &[TemporalEvaluationInput],
) -> TemporalValue {
    evaluate_temporal_program(program, inputs, TemporalEvaluationLimits::default()).unwrap()
}

pub(super) fn unary(
    operation: TemporalUnaryOperation,
    value: TemporalValue,
    result_type: TemporalType,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let nodes = vec![
        literal(0, value),
        TemporalNode {
            id: TemporalNodeId::new(1),
            value_type: result_type,
            kind: TemporalNodeKind::Unary {
                operation,
                operand: TemporalNodeId::new(0),
            },
            provenance_id: None,
        },
    ];
    evaluate_temporal_program(
        &program(Vec::new(), nodes, 1, result_type),
        &[],
        TemporalEvaluationLimits::default(),
    )
}

pub(super) fn binary(
    operation: TemporalBinaryOperation,
    left: TemporalValue,
    right: TemporalValue,
    result_type: TemporalType,
) -> Result<TemporalValue, TemporalEvaluationError> {
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
    evaluate_temporal_program(
        &program(Vec::new(), nodes, 2, result_type),
        &[],
        TemporalEvaluationLimits::default(),
    )
}

pub(super) fn compare(
    operation: TemporalCompareOperation,
    left: TemporalValue,
    right: TemporalValue,
) -> TemporalValue {
    compare_result(operation, left, right).unwrap()
}

pub(super) fn compare_result(
    operation: TemporalCompareOperation,
    left: TemporalValue,
    right: TemporalValue,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let nodes = vec![
        literal(0, left),
        literal(1, right),
        TemporalNode {
            id: TemporalNodeId::new(2),
            value_type: TemporalType::Boolean,
            kind: TemporalNodeKind::Compare {
                operation,
                left: TemporalNodeId::new(0),
                right: TemporalNodeId::new(1),
            },
            provenance_id: None,
        },
    ];
    evaluate_temporal_program(
        &program(Vec::new(), nodes, 2, TemporalType::Boolean),
        &[],
        TemporalEvaluationLimits::default(),
    )
}
