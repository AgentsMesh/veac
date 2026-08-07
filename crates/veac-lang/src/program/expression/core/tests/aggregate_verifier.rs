use super::{raw, raw_with_types, verify_error};
use crate::program::expression::{
    CoreInstruction, CoreInstructionKind, InputId, PrimitiveType, TypeEnvironment, Value, ValueType,
};

const MAP: &str = "map([1, 2], fn(value: int) -> int effect pure { value + 1 })";

#[test]
fn rejects_invalid_initial_and_callable_operands_before_runtime() {
    let (mut initial, functions) = raw(MAP);
    let iterable = aggregate(&initial).kind.operands().next().unwrap();
    let CoreInstructionKind::Collection { initial: value, .. } =
        &mut aggregate_mut(&mut initial).kind
    else {
        unreachable!()
    };
    *value = Some(iterable);
    assert_eq!(
        verify_error(initial, &functions).code(),
        "EXPRESSION_CORE_VERIFY"
    );

    let (mut callable, functions) = raw(MAP);
    let iterable = aggregate(&callable).kind.operands().next().unwrap();
    let CoreInstructionKind::Collection {
        callable: value, ..
    } = &mut aggregate_mut(&mut callable).kind
    else {
        unreachable!()
    };
    *value = iterable;
    assert_eq!(
        verify_error(callable, &functions).code(),
        "EXPRESSION_CORE_VERIFY"
    );
}

#[test]
fn rejects_declared_type_and_exact_metadata_corruption() {
    let (mut declared, functions) = raw(MAP);
    let integer_type = declared.blocks[0]
        .instructions
        .iter()
        .find(|value| matches!(value.kind, CoreInstructionKind::Literal(Value::Integer(_))))
        .unwrap()
        .type_id;
    aggregate_mut(&mut declared).type_id = integer_type;
    assert_eq!(
        verify_error(declared, &functions).code(),
        "EXPRESSION_CORE_VERIFY"
    );

    let (mut metadata, functions) = raw(MAP);
    aggregate_mut(&mut metadata)
        .metadata
        .absorb_shape(&crate::program::expression::CoreValueMetadata::constant());
    aggregate_mut(&mut metadata).metadata.leaf_stage = crate::program::expression::Stage::Temporal;
    assert_eq!(
        verify_error(metadata, &functions).code(),
        "EXPRESSION_CORE_VERIFY"
    );
}

#[test]
fn map_and_filter_metadata_keep_shape_and_leaf_dependencies_distinct() {
    let types = [("values".to_owned(), ValueType::list(integer()).unwrap())]
        .into_iter()
        .collect::<TypeEnvironment>();
    let (map, _) = raw_with_types(
        "map(values, fn(value: int) -> int effect pure { value + 1 })",
        &types,
    );
    let metadata = &aggregate(&map).metadata;
    assert_eq!(
        metadata.shape_dependencies().shape_input_ids(),
        [InputId::new(0)]
    );
    assert!(metadata.shape_dependencies().leaf_input_ids().is_empty());
    assert_eq!(
        metadata.leaf_dependencies().leaf_input_ids(),
        [InputId::new(0)]
    );

    let (filter, _) = raw_with_types(
        "filter(values, fn(value: int) -> bool effect pure { value > 0 })",
        &types,
    );
    let metadata = &aggregate(&filter).metadata;
    assert_eq!(
        metadata.shape_dependencies().shape_input_ids(),
        [InputId::new(0)]
    );
    assert_eq!(
        metadata.shape_dependencies().leaf_input_ids(),
        [InputId::new(0)]
    );
}

fn integer() -> ValueType {
    ValueType::primitive(PrimitiveType::Integer)
}

fn aggregate(program: &super::super::CoreProgram) -> &CoreInstruction {
    program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Collection { .. }))
        .unwrap()
}

fn aggregate_mut(program: &mut super::super::CoreProgram) -> &mut CoreInstruction {
    program
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Collection { .. }))
        .unwrap()
}
