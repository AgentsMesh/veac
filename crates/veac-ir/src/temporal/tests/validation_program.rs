use crate::*;

use super::support::{codes, literal, program, progress_program, scalar, seal};

fn assert_code(program: &TemporalProgram, expected: &str) {
    let error = validate_temporal_program(program).unwrap_err();
    assert!(
        codes(error).iter().any(|code| code == expected),
        "missing {expected}"
    );
}

#[test]
fn valid_program_passes_the_standalone_guard() {
    validate_temporal_program(&progress_program()).unwrap();
}

#[test]
fn program_header_and_digest_are_guarded() {
    let mut value = progress_program();
    value.opset_version += 1;
    assert_code(&value, "TEMPORAL_OPSET");

    let mut value = progress_program();
    value.id = serde_json::from_str::<TemporalProgramId>("\"bad\"").unwrap();
    value.provenance_id = serde_json::from_str::<TemporalProvenanceId>("\"bad\"").unwrap();
    assert_code(&value, "TEMPORAL_PROGRAM_ID");
    assert_code(&value, "TEMPORAL_PROVENANCE_ID");

    let mut value = progress_program();
    value.content_sha256 = "0".repeat(64);
    assert_code(&value, "TEMPORAL_PROGRAM_DIGEST");
}

#[test]
fn input_order_source_identity_and_clock_types_are_exact() {
    let mut value = progress_program();
    value.inputs[0].id = TemporalInputId::new(3);
    seal(&mut value);
    assert_code(&value, "TEMPORAL_INPUT_ORDER");

    let mut value = progress_program();
    value.inputs[0].value_type = TemporalType::Time;
    seal(&mut value);
    assert_code(&value, "TEMPORAL_INPUT_TYPE");

    let mut value = progress_program();
    value.inputs.push(TemporalInputDeclaration {
        id: TemporalInputId::new(1),
        value_type: TemporalType::Scalar,
        source: TemporalInputSource::Clock {
            clock: TemporalClock::Progress,
        },
    });
    seal(&mut value);
    assert_code(&value, "TEMPORAL_INPUT_SOURCE_DUPLICATE");

    let mut value = progress_program();
    value.inputs[0].source = TemporalInputSource::Parameter {
        parameter_id: serde_json::from_str("\"bad\"").unwrap(),
    };
    seal(&mut value);
    assert_code(&value, "TEMPORAL_PARAMETER_ID");
}

#[test]
fn node_topology_references_and_result_types_are_exact() {
    let mut value = progress_program();
    value.nodes[1].id = TemporalNodeId::new(7);
    seal(&mut value);
    assert_code(&value, "TEMPORAL_NODE_ORDER");

    let mut value = progress_program();
    let TemporalNodeKind::Binary { left, .. } = &mut value.nodes[2].kind else {
        unreachable!()
    };
    *left = TemporalNodeId::new(2);
    seal(&mut value);
    assert_code(&value, "TEMPORAL_NODE_REFERENCE");

    let mut value = progress_program();
    value.nodes[2].value_type = TemporalType::Integer;
    seal(&mut value);
    assert_code(&value, "TEMPORAL_NODE_TYPE");

    let mut value = progress_program();
    value.result = TemporalNodeId::new(99);
    seal(&mut value);
    assert_code(&value, "TEMPORAL_PROGRAM_RESULT");

    let mut value = progress_program();
    value.nodes.clear();
    seal(&mut value);
    assert_code(&value, "TEMPORAL_PROGRAM_NODE_COUNT");
}

#[test]
fn operation_contracts_reject_incompatible_operands() {
    let nodes = vec![
        literal(0, scalar(1.0)),
        literal(1, TemporalValue::Boolean { value: true }),
        TemporalNode {
            id: TemporalNodeId::new(2),
            value_type: TemporalType::Scalar,
            kind: TemporalNodeKind::Binary {
                operation: TemporalBinaryOperation::Add,
                left: TemporalNodeId::new(0),
                right: TemporalNodeId::new(1),
            },
            provenance_id: None,
        },
    ];
    assert_code(
        &program(Vec::new(), nodes, 2, TemporalType::Scalar),
        "TEMPORAL_OPERATION_TYPE",
    );
}

#[test]
fn every_literal_must_be_a_canonical_bounded_value() {
    let values = [
        TemporalValue::Integer {
            value: i64::try_from(MAX_SAFE_INTEGER + 1).unwrap(),
        },
        TemporalValue::Scalar { value: -0.0 },
        TemporalValue::Time {
            value: RationalTime {
                value: 0,
                timescale: 0,
            },
        },
        TemporalValue::Length {
            value: Length {
                value: f64::INFINITY,
                unit: LengthUnit::Pixels,
            },
        },
        TemporalValue::Angle { degrees: f64::NAN },
        TemporalValue::Vec2 {
            value: Vec2 {
                x: 0.0,
                y: f64::INFINITY,
            },
        },
    ];
    for value in values {
        let mut candidate = program(
            Vec::new(),
            vec![literal(0, scalar(0.0))],
            0,
            TemporalType::Scalar,
        );
        candidate.result_type = value.value_type();
        candidate.nodes[0] = literal(0, value);
        assert_code(&candidate, "TEMPORAL_VALUE");
    }
    let text = TemporalValue::Text {
        value: "x".repeat(MAX_TEMPORAL_TEXT_BYTES + 1),
    };
    let mut candidate = program(
        Vec::new(),
        vec![literal(0, scalar(0.0))],
        0,
        TemporalType::Scalar,
    );
    candidate.result_type = TemporalType::Text;
    candidate.nodes[0] = literal(0, text);
    assert_code(&candidate, "TEMPORAL_VALUE");
}
