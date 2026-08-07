use super::super::metadata::EffectEvidence;
use super::{raw, raw_with_types, verify_error};
use crate::program::expression::{
    ClosureDefinitionId, CoreDigest, CoreInstruction, CoreInstructionKind, Effect, FunctionEffect,
    FunctionMap, PrimitiveType, TypeEnvironment, ValueType, CORE_VERSION, MAX_FUNCTION_PARAMETERS,
};

#[test]
fn rejects_non_dense_and_unknown_closure_definition_ids() {
    let (mut dense, functions) = raw("fn() -> int effect pure { 1 }");
    dense.closure_definitions[0].id = ClosureDefinitionId::new(1);
    assert_message(dense, &functions, "closure definition IDs must be dense");

    let (mut unknown, functions) = raw("fn() -> int effect pure { 1 }");
    let CoreInstructionKind::Closure { definition, .. } = &mut closure(&mut unknown).kind else {
        unreachable!()
    };
    *definition = ClosureDefinitionId::new(9);
    assert_message(unknown, &functions, "unknown closure definition 9");
}

#[test]
fn rejects_closure_summary_digest_and_recursive_body_corruption() {
    let (mut summary, functions) = raw("fn() -> int effect pure { 1 }");
    summary.closure_definitions[0].summary.effect = EffectEvidence::from_effect(Effect::GraphEmit);
    assert_message(
        summary,
        &functions,
        "closure summary does not match its verified body",
    );

    let (mut digest, functions) = raw("fn() -> int effect pure { 1 }");
    digest.closure_definitions[0].digest = CoreDigest::from_bytes([0xff; 32]);
    assert_message(
        digest,
        &functions,
        "closure digest does not match its verified body",
    );

    let (mut escape_flag, functions) = raw("fn() -> int effect pure { 1 }");
    escape_flag.closure_definitions[0].non_escaping = true;
    assert_message(
        escape_flag,
        &functions,
        "closure digest does not match its verified body",
    );

    let (mut version, functions) = raw("fn() -> int effect pure { 1 }");
    version.closure_definitions[0].body.version = CORE_VERSION + 1;
    assert_message(
        version,
        &functions,
        &format!("unsupported Core version {}", CORE_VERSION + 1),
    );
}

#[test]
fn rejects_external_inputs_and_invalid_body_bindings_before_runtime() {
    let types = [(
        "outside".to_owned(),
        ValueType::primitive(PrimitiveType::Integer),
    )]
    .into_iter()
    .collect::<TypeEnvironment>();
    let (mut input, functions) = raw_with_types(
        "{ let ignored = outside; fn() -> int effect pure { 1 } }",
        &types,
    );
    input.closure_definitions[0]
        .body
        .inputs
        .push(input.inputs[0].clone());
    assert_message(
        input,
        &functions,
        "closure body must use explicit captures instead of external inputs",
    );

    let (mut capture, functions) = raw("{ let value = 1; fn() -> int effect pure { value } }");
    capture.closure_definitions[0].body.blocks[0].instructions[0].kind =
        CoreInstructionKind::Capture(4);
    assert_message(capture, &functions, "unknown capture index 4");

    let (mut parameter, functions) = raw("fn(value: int) -> int effect pure { value }");
    parameter.closure_definitions[0].body.blocks[0].instructions[0].kind =
        CoreInstructionKind::Parameter(4);
    assert_message(parameter, &functions, "unknown parameter index 4");
}

#[test]
fn rejects_oversized_or_escaping_function_capture_contracts() {
    let (mut arity, functions) = raw("fn() -> int effect pure { 1 }");
    arity.closure_definitions[0].parameter_types =
        vec![ValueType::primitive(PrimitiveType::Integer); MAX_FUNCTION_PARAMETERS + 1];
    assert_message(
        arity,
        &functions,
        "closure signature exceeds its arity limit",
    );

    let (mut escaping, functions) = raw("fn() -> int effect pure { 1 }");
    escaping.closure_definitions[0].capture_types = vec![ValueType::function(
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap()];
    assert_message(
        escaping,
        &functions,
        "escaping closure cannot capture function values",
    );
}

#[test]
fn closure_digest_is_stable_and_tracks_body_semantics() {
    let (first, _) = raw("fn(value: int) -> int effect pure { value + 1 }");
    let (same, _) = raw("fn(value: int) -> int effect pure { value + 1 }");
    let (different, _) = raw("fn(value: int) -> int effect pure { value + 2 }");
    let (different_effect, _) = raw("fn(value: int) -> int effect any { value + 1 }");
    assert_eq!(
        first.closure_definitions[0].digest(),
        same.closure_definitions[0].digest()
    );
    assert_ne!(
        first.closure_definitions[0].digest(),
        different.closure_definitions[0].digest()
    );
    assert_ne!(
        first.closure_definitions[0].digest(),
        different_effect.closure_definitions[0].digest()
    );
    assert!(!first.closure_definitions[0].non_escaping());
}

#[test]
fn rejects_declared_effects_that_understate_verified_body_evidence() {
    let local_source = "fn() -> int effect local { var value = 0; set value = 1; value }";
    for contract in [FunctionEffect::Pure, FunctionEffect::Emit] {
        let (mut program, functions) = raw(local_source);
        program.closure_definitions[0].effect = contract;
        assert_message(
            program,
            &functions,
            &format!("closure body effect does not satisfy declared effect {contract}"),
        );
    }

    let emit_source = "fn() -> Sequence effect emit { sequence(identifier(\"effect-proof\"), \
        \"effect proof\", sequence_settings(canvas(16px, 16px), frame_rate(1, 1), 8000)) }";
    for contract in [FunctionEffect::Pure, FunctionEffect::Local] {
        let (mut program, functions) = raw(emit_source);
        program.closure_definitions[0].effect = contract;
        assert_message(
            program,
            &functions,
            &format!("closure body effect does not satisfy declared effect {contract}"),
        );
    }
}

#[test]
fn any_contract_accepts_combined_local_mutation_and_graph_emit_evidence() {
    let source = "fn() -> Sequence effect any { var key = identifier(\"first\"); \
        set key = identifier(\"second\"); sequence(key, \"effect proof\", \
        sequence_settings(canvas(16px, 16px), frame_rate(1, 1), 8000)) }";
    let (program, _) = raw(source);
    let summary = program.closure_definitions[0].summary();
    assert_eq!(program.closure_definitions[0].effect(), FunctionEffect::Any);
    assert_eq!(summary.effect(), Effect::GraphEmit);
    assert!(summary.contains_local_mutation());
}

fn closure(program: &mut super::super::CoreProgram) -> &mut CoreInstruction {
    program
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Closure { .. }))
        .unwrap()
}

fn assert_message(program: super::super::CoreProgram, functions: &FunctionMap, message: &str) {
    assert_eq!(verify_error(program, functions).message(), message);
}
