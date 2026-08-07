use crate::*;

fn input_program() -> (TemporalProgram, TemporalValue) {
    let value = TemporalValue::Text {
        value: "budget".repeat(8),
    };
    let value_type = value.value_type();
    let program = super::support::program(
        vec![TemporalInputDeclaration {
            id: TemporalInputId::new(0),
            value_type,
            source: TemporalInputSource::Parameter {
                parameter_id: TemporalParameterId::new("tpm_text").unwrap(),
            },
        }],
        vec![TemporalNode {
            id: TemporalNodeId::new(0),
            value_type,
            kind: TemporalNodeKind::Input {
                input_id: TemporalInputId::new(0),
            },
            provenance_id: None,
        }],
        0,
        value_type,
    );
    (program, value)
}

fn evaluate(max_value_bytes: usize) -> TemporalEvaluationError {
    let (program, value) = input_program();
    evaluate_temporal_program(
        &program,
        &[TemporalEvaluationInput {
            input_id: TemporalInputId::new(0),
            value,
        }],
        TemporalEvaluationLimits {
            max_steps: 1,
            max_value_bytes,
        },
    )
    .unwrap_err()
}

#[test]
fn input_admission_is_charged_before_evaluation_storage() {
    let (_, value) = input_program();
    let error = evaluate(value.logical_bytes() - 1);
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_LIMIT");
    assert_eq!(error.pointer(), "/inputs/0");
}

#[test]
fn input_node_storage_is_reserved_before_cloning() {
    let (_, value) = input_program();
    let error = evaluate(value.logical_bytes() * 2 - 1);
    assert_eq!(error.code(), "TEMPORAL_EVALUATION_LIMIT");
    assert_eq!(error.pointer(), "/program/nodes/0");
}
