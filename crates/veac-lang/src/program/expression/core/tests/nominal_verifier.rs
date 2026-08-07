use super::nominal_fixture::{matching, projection, structure};
use super::{empty_registry, verify_error};
use crate::program::expression::{CoreInstructionKind, CoreTerminator, CoreTypeId};
use crate::program::VariantIndex;

#[test]
fn verifies_nominal_construction_projection_and_exhaustive_match() {
    let registry = empty_registry();
    assert!(super::super::verify(structure(), &registry, &[]).is_ok());
    assert!(super::super::verify(matching(), &registry, &[]).is_ok());
    assert!(super::super::verify(projection(), &registry, &[]).is_ok());
}

#[test]
fn runtime_binds_only_the_selected_match_payload() {
    let registry = std::sync::Arc::new(empty_registry());
    let verified = super::super::verify(matching(), &registry, &[]).unwrap();
    let compiled = crate::program::expression::CompiledExpression::new(verified, registry);
    let value = crate::program::expression::evaluate_compiled(
        &compiled,
        &crate::program::expression::Environment::new(),
    )
    .unwrap();
    assert_eq!(value, crate::program::expression::Value::Integer(9));
}

#[test]
fn rejects_missing_extra_and_unordered_definition_tables() {
    let functions = crate::program::expression::FunctionMap::new();
    let mut missing = structure();
    missing.nominal_definitions.clear();
    assert!(verify_error(missing, &functions)
        .message()
        .contains("unknown TypeId"));

    let mut extra = structure();
    let duplicate = extra.nominal_definitions[0].clone();
    extra.nominal_definitions.push(duplicate);
    assert!(verify_error(extra, &functions)
        .message()
        .contains("unique and ordered"));
}

#[test]
fn rejects_constructor_type_layout_and_variant_corruption() {
    let functions = crate::program::expression::FunctionMap::new();
    let mut fields = structure();
    let CoreInstructionKind::StructConstruct { fields: values, .. } =
        &mut fields.blocks[0].instructions[1].kind
    else {
        unreachable!()
    };
    values.clear();
    assert!(verify_error(fields, &functions)
        .message()
        .contains("field count"));

    let mut variant = matching();
    let CoreInstructionKind::EnumConstruct { variant: index, .. } =
        &mut variant.blocks[0].instructions[1].kind
    else {
        unreachable!()
    };
    *index = VariantIndex::new(9);
    assert!(verify_error(variant, &functions)
        .message()
        .contains("unknown variant"));
}

#[test]
fn rejects_non_exhaustive_reordered_and_payload_mismatched_match() {
    let functions = crate::program::expression::FunctionMap::new();
    let mut missing = matching();
    let CoreTerminator::Match { arms, .. } = &mut missing.blocks[0].terminator else {
        unreachable!()
    };
    arms.pop();
    missing.blocks[2].terminator = CoreTerminator::Jump {
        target: crate::program::expression::BlockId::new(3),
        arguments: Vec::new(),
        span: 0..1,
    };
    let error = verify_error(missing, &functions);
    assert!(
        error.message().contains("exactly one arm"),
        "{}",
        error.message()
    );

    let mut reordered = matching();
    let CoreTerminator::Match { arms, .. } = &mut reordered.blocks[0].terminator else {
        unreachable!()
    };
    arms.swap(0, 1);
    assert!(verify_error(reordered, &functions)
        .message()
        .contains("ordered by variant"));

    let mut payload = matching();
    payload.blocks[1].parameters[0].type_id = CoreTypeId::new(2);
    let error = verify_error(payload, &functions);
    assert!(
        error.message().contains("expects int, found text"),
        "{}",
        error.message()
    );
}
