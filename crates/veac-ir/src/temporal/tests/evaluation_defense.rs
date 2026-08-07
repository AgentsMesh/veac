use std::collections::BTreeMap;

use crate::*;

use super::support::scalar;

fn direct(kind: TemporalNodeKind, values: &[TemporalValue]) -> TemporalEvaluationError {
    super::super::evaluation::test_support::evaluate_node(&kind, values, &BTreeMap::new(), "/node")
        .unwrap_err()
}

#[test]
fn evaluator_defends_verified_operand_and_input_invariants() {
    let missing_input = direct(
        TemporalNodeKind::Input {
            input_id: TemporalInputId::new(0),
        },
        &[],
    );
    assert_eq!(missing_input.code(), "TEMPORAL_EVALUATION_INPUT");

    let missing_operand = direct(
        TemporalNodeKind::Unary {
            operation: TemporalUnaryOperation::Negate,
            operand: TemporalNodeId::new(0),
        },
        &[],
    );
    assert_eq!(missing_operand.code(), "TEMPORAL_EVALUATION_CONTRACT");
}

#[test]
fn evaluator_defends_control_and_vector_value_invariants() {
    let values = [scalar(1.0)];
    let select = direct(
        TemporalNodeKind::Select {
            condition: TemporalNodeId::new(0),
            when_true: TemporalNodeId::new(0),
            when_false: TemporalNodeId::new(0),
        },
        &values,
    );
    assert_eq!(select.code(), "TEMPORAL_EVALUATION_CONTRACT");
    let compose = direct(
        TemporalNodeKind::ComposeVec2 {
            x: TemporalNodeId::new(0),
            y: TemporalNodeId::new(1),
        },
        &[TemporalValue::Boolean { value: true }, scalar(1.0)],
    );
    assert_eq!(compose.code(), "TEMPORAL_EVALUATION_CONTRACT");
    let project = direct(
        TemporalNodeKind::ProjectVec2 {
            value: TemporalNodeId::new(0),
            axis: TemporalVectorAxis::X,
        },
        &values,
    );
    assert_eq!(project.code(), "TEMPORAL_EVALUATION_CONTRACT");
}

#[test]
fn evaluator_defends_curve_driver_and_unit_invariants() {
    let keys = vec![TemporalCurveKey {
        position: TemporalCurvePosition::Scalar { value: 0.0 },
        value: scalar(0.0),
        interpolation: Interpolation::Hold,
    }];
    let error = direct(
        TemporalNodeKind::CurveSample {
            input: TemporalNodeId::new(0),
            keys,
        },
        &[TemporalValue::Boolean { value: true }],
    );
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_CONTRACT");

    let keys = vec![
        TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value: 0.0 },
            value: TemporalValue::Length {
                value: Length {
                    value: 0.0,
                    unit: LengthUnit::Pixels,
                },
            },
            interpolation: Interpolation::Linear,
        },
        TemporalCurveKey {
            position: TemporalCurvePosition::Scalar { value: 1.0 },
            value: TemporalValue::Length {
                value: Length {
                    value: 1.0,
                    unit: LengthUnit::Percent,
                },
            },
            interpolation: Interpolation::Hold,
        },
    ];
    let error = direct(
        TemporalNodeKind::CurveSample {
            input: TemporalNodeId::new(0),
            keys,
        },
        &[scalar(0.5)],
    );
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_VALUE");
}
