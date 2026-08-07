use std::sync::Arc;

use super::nominal_fixture::{matching, projection, structure};
use super::verify_error;
use crate::program::expression::{
    compile_expression, BlockId, CoreInstructionKind, CoreTerminator, CoreValueMetadata,
    ExpressionContext, FunctionMap, PrimitiveType, TypeEnvironment, Value, ValueId, ValueType,
};
use crate::program::{
    FieldDefinition, FieldIndex, StructDefinition, TypeDefinition, TypeDefinitionKind,
    TypeRegistryBuilder,
};

#[test]
fn rejects_stale_nominal_literal_definition_digest() {
    let mut program = structure();
    let current = program.nominal_definitions[0].definition();
    let stale = Arc::new(TypeDefinition::new(
        current.canonical_source_id(),
        current.declared_name(),
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "renamed",
            crate::program::expression::ValueType::primitive(
                crate::program::expression::PrimitiveType::Integer,
            ),
        )])),
    ));
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&stale)).unwrap();
    let value = Value::structure(
        &builder.finish().unwrap(),
        stale.type_ref().id(),
        vec![Value::Integer(7)],
    )
    .unwrap();
    program.blocks[0].instructions[1].kind = CoreInstructionKind::Literal(value);
    assert_message(program, "nominal value layout does not match");
}

#[test]
fn rejects_struct_projection_receiver_and_field_corruption() {
    let mut receiver = projection();
    let CoreInstructionKind::StructProject { structure, .. } =
        &mut receiver.blocks[0].instructions[2].kind
    else {
        unreachable!()
    };
    *structure = ValueId::new(0);
    assert_message(receiver, "StructProject receiver must be nominal");

    let mut field = projection();
    let CoreInstructionKind::StructProject { field: index, .. } =
        &mut field.blocks[0].instructions[2].kind
    else {
        unreachable!()
    };
    *index = FieldIndex::new(9);
    assert_message(field, "StructProject has an unknown field index");
}

#[test]
fn rejects_duplicate_and_nonexclusive_match_targets() {
    let mut duplicate = matching();
    let CoreTerminator::Match { arms, .. } = &mut duplicate.blocks[0].terminator else {
        unreachable!()
    };
    arms[2].target = arms[1].target;
    duplicate.blocks[1].terminator = CoreTerminator::Jump {
        target: BlockId::new(3),
        arguments: Vec::new(),
        span: 0..1,
    };
    assert_message(
        duplicate,
        "match arm target must have exactly one exclusive predecessor",
    );

    let mut shared = matching();
    shared.blocks[3].terminator = CoreTerminator::Jump {
        target: BlockId::new(1),
        arguments: vec![ValueId::new(8)],
        span: 0..1,
    };
    assert_message(
        shared,
        "match arm target must have exactly one exclusive predecessor",
    );
}

#[test]
fn rejects_definition_table_order_corruption() {
    let mut program = structure();
    let extra = Arc::new(TypeDefinition::new(
        "other.veac",
        "Marker",
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    ));
    program
        .nominal_definitions
        .push(crate::program::expression::CoreNominalDefinition::new(
            extra,
            0..1,
        ));
    program
        .nominal_definitions
        .sort_by_key(|entry| entry.definition().type_ref().id());
    program.nominal_definitions.reverse();
    assert_message(
        program,
        "Core nominal definitions must be unique and ordered by TypeId",
    );
}

#[test]
fn rejects_nominal_field_metadata_tree_corruption() {
    let mut constructor = structure();
    constructor.blocks[0].instructions[1].metadata = CoreValueMetadata::constant();
    assert_message(
        constructor,
        "instruction metadata does not match its operands",
    );

    let mut payload = matching();
    payload.blocks[2].parameters[0].metadata = CoreValueMetadata::constant();
    assert_message(
        payload,
        "block parameter metadata is not the exact control/data dependency union",
    );
}

#[test]
fn rejects_callable_contract_removed_from_raw_field_projection() {
    let context = callback_context();
    let mut program = compile_expression(
        "CallbackBox { callback: fn() -> int effect pure { 1 } }.callback()",
        &TypeEnvironment::new(),
        &context,
    )
    .unwrap()
    .core()
    .clone();
    let projection = program.blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::StructProject { .. }))
        .unwrap();
    projection.metadata.callable = None;
    assert_message(program, "instruction metadata does not match its operands");
}

fn callback_context() -> ExpressionContext {
    let callback = ValueType::function(
        Vec::new(),
        PrimitiveType::Integer.into(),
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "CallbackBox",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "callback",
            callback,
        )])),
    ));
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&definition)).unwrap();
    builder
        .bind("CallbackBox", definition.type_ref().clone())
        .unwrap();
    ExpressionContext::empty().with_types(Arc::new(builder.finish().unwrap()))
}

fn assert_message(program: crate::program::expression::CoreProgram, message: &str) {
    let functions = FunctionMap::new();
    let error = verify_error(program, &functions);
    assert!(
        error.message().contains(message),
        "expected `{message}`, found `{}`",
        error.message()
    );
}
