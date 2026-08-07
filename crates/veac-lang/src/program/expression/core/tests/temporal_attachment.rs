use std::collections::BTreeSet;

use super::{raw_with_types, verify_error};
use crate::program::expression::{
    compile_expression, CoreInstructionKind, CoreValueMetadata, Effect, ExpressionContext,
    PrimitiveType, Stage, TypeEnvironment, ValueType,
};
use crate::program::DomainType;

mod cases;
mod stage;
use cases::cases;

#[test]
fn all_twenty_six_closed_property_target_pairs_lower_to_verified_core() {
    let mut opcodes = BTreeSet::new();
    for (source, result, apply) in cases() {
        let program = raw_with_types(&source, &types(result, apply)).0;
        let instruction = attachment(&program);
        let CoreInstructionKind::TemporalAttach { kind, .. } = instruction.kind() else {
            unreachable!()
        };
        opcodes.insert(*kind);
        assert_eq!(instruction.metadata().effect(), Effect::GraphEmit);
        assert_eq!(instruction.metadata().shape_stage(), Stage::Build);
        assert_eq!(program.result_type().as_domain(), Some(owner(apply)));
    }
    assert_eq!(opcodes.len(), 26);
}

#[test]
fn surface_rejects_invalid_pair_owner_selector_and_result_types() {
    let failures = [
        (
            "animate visual-opacity on text(owner) { 0.5 }",
            types(scalar(), false),
            "cannot target",
        ),
        (
            "animate visual-opacity on clip(owner) { 0.5 }",
            types(scalar(), true),
            "expects Item",
        ),
        (
            "animate mask-feather on clip-mask(owner, id) { 0.5 }",
            types(scalar(), false),
            "expects int",
        ),
        (
            "animate visual-opacity on clip(owner) { point(0px, 0px) }",
            types(scalar(), false),
            "expects scalar",
        ),
    ];
    for (source, mut types, expected) in failures {
        types.insert("point_output".into(), ValueType::domain(DomainType::Point));
        let error = compile_expression(source, &types, &ExpressionContext::empty()).unwrap_err();
        assert!(error.to_string().contains(expected), "{source}: {error}");
    }
}

#[test]
fn verifier_rejects_corrupt_kind_operands_result_and_metadata() {
    let source = "animate mask-feather on clip-mask(owner, index) { 0.5 }";
    let types = types(scalar(), false);
    let (program, functions) = raw_with_types(source, &types);

    let mut kind = program.clone();
    let CoreInstructionKind::TemporalAttach { kind: value, .. } =
        &mut attachment_mut(&mut kind).kind
    else {
        unreachable!()
    };
    *value = u8::MAX;
    assert!(verify_error(kind, &functions)
        .message()
        .contains("unknown temporal"));

    let mut owner = program.clone();
    let CoreInstructionKind::TemporalAttach {
        owner: value,
        selectors,
        ..
    } = &mut attachment_mut(&mut owner).kind
    else {
        unreachable!()
    };
    *value = selectors[0];
    assert!(verify_error(owner, &functions)
        .message()
        .contains("expects Item"));

    let mut arity = program.clone();
    let CoreInstructionKind::TemporalAttach { selectors, .. } =
        &mut attachment_mut(&mut arity).kind
    else {
        unreachable!()
    };
    selectors.clear();
    assert!(verify_error(arity, &functions)
        .message()
        .contains("selector arity"));

    let mut animation = program.clone();
    let CoreInstructionKind::TemporalAttach {
        owner,
        animation: animation_id,
        ..
    } = &mut attachment_mut(&mut animation).kind
    else {
        unreachable!()
    };
    *animation_id = *owner;
    let error = verify_error(animation, &functions);
    assert!(error.message().contains("parameter stages"), "{error}");

    let mut metadata = program;
    attachment_mut(&mut metadata).metadata = CoreValueMetadata::constant();
    assert!(verify_error(metadata, &functions)
        .message()
        .contains("metadata"));
}

fn attachment(program: &super::super::CoreProgram) -> &super::super::CoreInstruction {
    program.blocks[0]
        .instructions
        .iter()
        .find(|value| matches!(value.kind(), CoreInstructionKind::TemporalAttach { .. }))
        .unwrap()
}

fn attachment_mut(program: &mut super::super::CoreProgram) -> &mut super::super::CoreInstruction {
    program.blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind(), CoreInstructionKind::TemporalAttach { .. }))
        .unwrap()
}

fn types(result: ValueType, apply: bool) -> TypeEnvironment {
    TypeEnvironment::from([
        ("owner".into(), ValueType::domain(owner(apply))),
        ("output".into(), result),
        ("index".into(), PrimitiveType::Integer.into()),
        ("id".into(), PrimitiveType::Identifier.into()),
    ])
}

fn owner(apply: bool) -> DomainType {
    if apply {
        DomainType::Apply
    } else {
        DomainType::Item
    }
}

fn scalar() -> ValueType {
    PrimitiveType::Scalar.into()
}
