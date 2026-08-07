use std::collections::BTreeMap;
use std::mem::{size_of, size_of_val};
use std::sync::Arc;

use super::super::super::{CoreProgram, VerifiedCoreProgram};
use super::super::{metadata, sequence_type_bytes};
use crate::program::expression::{
    compile_expression, evaluate, Environment, ExpressionContext, TypeEnvironment,
};

fn compiled(source: &str) -> crate::program::expression::CompiledExpression {
    compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap()
}

#[test]
fn raw_core_counts_closure_signatures_metadata_and_recursive_body() {
    let compiled =
        compiled("{ let offset = 1; fn(value: int) -> int effect pure { value + offset } }");
    let mut program = compiled.core().clone();
    let definition = &program.closure_definitions[0];
    let expected = size_of_val(definition)
        + sequence_type_bytes(&definition.parameter_types).unwrap()
        + definition.parameter_stages.len() * size_of::<crate::program::expression::Stage>()
        + sequence_type_bytes(&definition.capture_types).unwrap()
        + metadata::summary_payload(&definition.summary).unwrap()
        + definition.body.retained_bytes().unwrap();
    let with_definition = program.retained_bytes().unwrap();
    program.closure_definitions.clear();
    assert_eq!(
        with_definition - program.retained_bytes().unwrap(),
        expected
    );
}

#[test]
fn raw_core_counts_closure_callable_payload() {
    let compiled = compiled("fn(value: int) -> int effect pure { value + 1 }");
    let mut program = compiled.core().clone();
    let with_callable = program.retained_bytes().unwrap();
    program.blocks[0].instructions[0].metadata.callable = None;
    assert!(with_callable > program.retained_bytes().unwrap());
}

#[test]
fn raw_core_counts_trusted_callable_input_descriptor_payload() {
    let closure = evaluate(
        "{ let offset = 1; fn(value: int) -> int effect pure { value + offset } }",
        &Environment::new(),
    )
    .unwrap();
    let values = BTreeMap::from([("callback".to_owned(), Arc::new(closure))]);
    let compiled = crate::program::expression::compile::compile_with_values(
        "callback(1)",
        &values,
        &ExpressionContext::empty(),
    )
    .unwrap();

    let mut program = compiled.core().clone();
    let descriptor = program.inputs[0].callable.take().unwrap();
    let expected = sequence_type_bytes(&descriptor.capture_types).unwrap()
        + metadata::summary_payload(&descriptor.summary).unwrap();

    assert_eq!(
        compiled.core().retained_bytes().unwrap() - program.retained_bytes().unwrap(),
        expected,
    );
}

#[test]
fn verified_core_counts_parallel_closure_storage_and_recursive_body() {
    let compiled = compiled("fn(value: int) -> int effect pure { value + 1 }");
    let verified = compiled.verified();
    let raw_and_inline = size_of::<VerifiedCoreProgram>() - size_of::<CoreProgram>()
        + verified.core().retained_bytes().unwrap();
    let parallel = size_of::<std::sync::Arc<super::super::super::VerifiedClosureDefinition>>()
        + super::super::verified::closure_bytes(verified.closures()[0].as_ref()).unwrap();
    assert_eq!(
        verified.retained_bytes().unwrap(),
        raw_and_inline + parallel
    );
}
