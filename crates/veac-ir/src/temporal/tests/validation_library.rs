use crate::*;

use super::support::{codes, library, literal, program, provenance, provenance_id, scalar, seal};

fn assert_code(value: &TemporalProgramLibrary, expected: &str) {
    let actual = codes(validate_temporal_library(value).unwrap_err());
    assert!(
        actual.iter().any(|code| code == expected),
        "missing {expected}: {actual:?}"
    );
}

#[test]
fn valid_library_closes_program_binding_and_provenance_references() {
    validate_temporal_library(&library()).unwrap();
}

#[test]
fn library_versions_and_unique_identities_are_guarded() {
    let mut value = library();
    value.opset_version += 1;
    assert_code(&value, "TEMPORAL_OPSET");

    let mut value = library();
    value.programs.push(value.programs[0].clone());
    assert_code(&value, "TEMPORAL_PROGRAM_DUPLICATE");
    assert_code(&value, "TEMPORAL_PROGRAM_CONTENT_DUPLICATE");

    let mut value = library();
    value.bindings.push(value.bindings[0].clone());
    assert_code(&value, "TEMPORAL_BINDING_DUPLICATE");

    let mut value = library();
    value.provenance.push(value.provenance[0].clone());
    assert_code(&value, "TEMPORAL_PROVENANCE_DUPLICATE");
}

#[test]
fn all_provenance_references_must_resolve() {
    let mut value = library();
    value.programs[0].provenance_id = TemporalProvenanceId::new("tpv_missing").unwrap();
    value.bindings[0].provenance_id = TemporalProvenanceId::new("tpv_missing").unwrap();
    value.programs[0].nodes[0].provenance_id =
        Some(TemporalProvenanceId::new("tpv_missing").unwrap());
    seal(&mut value.programs[0]);
    assert_code(&value, "TEMPORAL_PROVENANCE_MISSING");

    let mut value = library();
    value.bindings[0].program_id = TemporalProgramId::new("tpg_missing").unwrap();
    assert_code(&value, "TEMPORAL_PROGRAM_MISSING");
}

#[test]
fn clock_bindings_require_exact_roles_owners_and_coverage() {
    let mut value = library();
    value.bindings[0].result_type = TemporalType::Time;
    assert_code(&value, "TEMPORAL_BINDING_RESULT");

    let mut value = library();
    value.bindings[0].clocks[0].clock = TemporalClock::Frame;
    assert_code(&value, "TEMPORAL_CLOCK_BINDING");

    let mut value = library();
    value.bindings[0].clocks[0].owner = TemporalClockOwner::Sequence {
        sequence_id: SequenceId::new("seq_main").unwrap(),
    };
    assert_code(&value, "TEMPORAL_CLOCK_BINDING");

    let mut value = library();
    value.bindings[0].clocks.clear();
    assert_code(&value, "TEMPORAL_BINDING_INCOMPLETE");

    let mut value = library();
    let duplicate = value.bindings[0].clocks[0].clone();
    value.bindings[0].clocks.push(duplicate);
    assert_code(&value, "TEMPORAL_INPUT_BINDING_DUPLICATE");
}

#[test]
fn parameter_bindings_are_typed_and_identity_pinned() {
    let mut value = parameter_library();
    validate_temporal_library(&value).unwrap();

    value.bindings[0].parameters[0].parameter_id = TemporalParameterId::new("tpm_other").unwrap();
    assert_code(&value, "TEMPORAL_PARAMETER_BINDING");
}

#[test]
fn canonical_schema_and_json_reject_the_removed_analysis_surface() {
    let schema = serde_json::to_string(&schemars::schema_for!(TemporalProgramLibrary)).unwrap();
    assert!(!schema.contains("\"analyses\""));
    assert!(!schema.contains("analysis_id"));

    let mut binding_json = serde_json::to_value(library()).unwrap();
    binding_json["bindings"][0]["analyses"] = serde_json::json!([]);
    assert!(serde_json::from_value::<TemporalProgramLibrary>(binding_json).is_err());

    let mut source_json = serde_json::to_value(library()).unwrap();
    source_json["programs"][0]["inputs"][0]["source"] =
        serde_json::json!({"type": "analysis", "analysis_id": "tan_removed"});
    assert!(serde_json::from_value::<TemporalProgramLibrary>(source_json).is_err());
}

#[test]
fn provenance_is_typed_bounded_and_source_located() {
    let mut value = library();
    value.provenance[0].definition.name.clear();
    value.provenance[0].origin.span = TemporalSourceSpan { start: 5, end: 4 };
    value.provenance[0].origin.definition_id = TemporalDefinitionId::new("def_other").unwrap();
    assert_code(&value, "TEMPORAL_DEFINITION_NAME");
    assert_code(&value, "TEMPORAL_SOURCE_SPAN");
    assert_code(&value, "TEMPORAL_PROVENANCE_ORIGIN");

    let mut value = library();
    let site = value.provenance[0].origin.clone();
    value.provenance[0].call_stack = vec![site; MAX_TEMPORAL_CALL_DEPTH + 1];
    assert_code(&value, "TEMPORAL_CALL_DEPTH");

    let mut value = library();
    value.provenance[0].logical_keys = vec![TemporalLogicalKey::new("key_same").unwrap(); 2];
    assert_code(&value, "TEMPORAL_LOGICAL_KEY_DUPLICATE");
}

fn parameter_library() -> TemporalProgramLibrary {
    let parameter_id = TemporalParameterId::new("tpm_gain").unwrap();
    let inputs = vec![TemporalInputDeclaration {
        id: TemporalInputId::new(0),
        value_type: TemporalType::Scalar,
        source: TemporalInputSource::Parameter {
            parameter_id: parameter_id.clone(),
        },
    }];
    let mut program = program(
        inputs,
        vec![literal(0, scalar(1.0))],
        0,
        TemporalType::Scalar,
    );
    program.id = TemporalProgramId::new("tpg_mixed").unwrap();
    seal(&mut program);
    let binding = TemporalBinding {
        id: TemporalBindingId::new("tbd_mixed").unwrap(),
        program_id: program.id.clone(),
        result_type: TemporalType::Scalar,
        clocks: Vec::new(),
        parameters: vec![TemporalParameterBinding {
            input_id: TemporalInputId::new(0),
            parameter_id,
            value: scalar(0.5),
        }],
        provenance_id: provenance_id(),
    };
    TemporalProgramLibrary {
        opset_version: TEMPORAL_OPSET_VERSION,
        programs: vec![program],
        bindings: vec![binding],
        provenance: vec![provenance()],
    }
}
