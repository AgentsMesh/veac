use super::{raw_with_types, types, verify_error};
use crate::program::expression::{FunctionEffect, Stage};

#[test]
fn attachment_closure_parameters_are_temporal_in_verified_core() {
    let (program, _) = raw_with_types(
        "animate visual-opacity on clip(owner) { progress }",
        &types(super::scalar(), false),
    );
    let definition = &program.closure_definitions[0];
    assert_eq!(definition.parameter_stages.len(), 5);
    assert!(definition
        .parameter_stages
        .iter()
        .all(|stage| *stage == Stage::Temporal));
}

#[test]
fn closure_stage_enters_digest_and_corruption_is_reverified() {
    let (mut program, functions) = raw_with_types(
        "animate visual-opacity on clip(owner) { progress }",
        &types(super::scalar(), false),
    );
    let definition = &program.closure_definitions[0];
    let changed = vec![Stage::Const; definition.parameter_stages.len()];
    assert_ne!(
        definition.digest,
        super::super::super::closure_digest(
            &definition.parameter_types,
            &changed,
            &definition.capture_types,
            FunctionEffect::Pure,
            definition.non_escaping,
            &definition.body,
        )
    );
    program.closure_definitions[0].parameter_stages = changed;
    assert!(verify_error(program, &functions)
        .message()
        .contains("parameter stages do not match"));
}
