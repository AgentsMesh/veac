use super::*;
use crate::program::expression::core::metadata::CallableContract;
use crate::program::expression::core::verify::definitions::Definitions;
use crate::program::expression::core::verify::test_support::raw;
use crate::program::expression::{
    CollectionOperation, CoreInstructionKind, CoreType, FunctionEffect, PrimitiveType, Value,
    ValueType,
};
use crate::program::TypeRegistry;

fn integer() -> ValueType {
    ValueType::primitive(PrimitiveType::Integer)
}

fn collection(program: &CoreProgram) -> CoreInstruction {
    program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Collection { .. }))
        .unwrap()
        .clone()
}

fn error(program: &CoreProgram, value: &CoreInstruction) -> String {
    let definitions = Definitions::collect(program).unwrap();
    verify(value, program, &definitions, &TypeRegistry::default())
        .unwrap_err()
        .message()
        .to_owned()
}

#[test]
fn iterable_contract_accepts_structural_collections_only() {
    let instruction = raw("1").0.blocks[0].instructions[0].clone();
    let list = ValueType::list(integer()).unwrap();
    let range = ValueType::range(integer()).unwrap();
    let map = ValueType::map(PrimitiveType::Text.into(), integer()).unwrap();
    assert_eq!(iterable_element(&list, &instruction).unwrap(), integer());
    assert_eq!(iterable_element(&range, &instruction).unwrap(), integer());
    assert!(matches!(
        iterable_element(&map, &instruction).unwrap().kind(),
        ValueTypeKind::Tuple(_)
    ));
    assert!(iterable_element(&integer(), &instruction).is_err());
}

#[test]
fn helper_contracts_reject_initials_and_bad_signatures() {
    let instruction = raw("1").0.blocks[0].instructions[0].clone();
    assert!(require_no_initial(Some(ValueId::new(0)), &instruction).is_err());
    assert!(require_no_initial(None, &instruction).is_ok());
    assert!(require_signature(&[integer()], &[PrimitiveType::Text.into()], &instruction).is_err());
    assert!(require_signature(&[integer()], &[integer()], &instruction).is_ok());
}

#[test]
fn verifier_rejects_non_collection_inputs_and_non_callable_callbacks() {
    let (program, _) = raw("map([1, 2], fn(value: int) -> int effect pure { value + 1 })");
    let mut value = collection(&program);
    let literal = program.blocks[0]
        .instructions
        .iter()
        .find(|value| matches!(value.kind, CoreInstructionKind::Literal(Value::Integer(_))))
        .unwrap()
        .id;
    let original_iterable = match &value.kind {
        CoreInstructionKind::Collection { iterable, .. } => *iterable,
        _ => unreachable!(),
    };
    let CoreInstructionKind::Collection { iterable, .. } = &mut value.kind else {
        unreachable!()
    };
    *iterable = literal;
    assert!(error(&program, &value).contains("input must be list"));

    let CoreInstructionKind::Collection {
        iterable, callable, ..
    } = &mut value.kind
    else {
        unreachable!()
    };
    *iterable = original_iterable;
    *callable = literal;
    assert!(error(&program, &value).contains("function type"));
}

#[test]
fn verifier_enforces_filter_and_fold_specific_contracts() {
    let (map, _) = raw("map([1, 2], fn(value: int) -> int effect pure { value + 1 })");
    let mut filter = collection(&map);
    let CoreInstructionKind::Collection { operation, .. } = &mut filter.kind else {
        unreachable!()
    };
    *operation = CollectionOperation::Filter;
    assert!(error(&map, &filter).contains("return bool"));

    let (fold, _) =
        raw("fold([1, 2], 0, fn(total: int, value: int) -> int effect pure { total + value })");
    let mut missing = collection(&fold);
    let CoreInstructionKind::Collection { initial, .. } = &mut missing.kind else {
        unreachable!()
    };
    *initial = None;
    assert!(error(&fold, &missing).contains("requires an initial"));
}

#[test]
fn fold_result_must_equal_its_accumulator_type() {
    let (mut program, _) =
        raw("fold([1, 2], 0, fn(total: int, value: int) -> int effect pure { total + value })");
    let value = collection(&program);
    let CoreInstructionKind::Collection { callable, .. } = &value.kind else {
        unreachable!()
    };
    let type_id = program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|value| value.id == *callable)
        .unwrap()
        .type_id;
    program.types.entries[type_id.index().unwrap()].kind = CoreType::Value(
        ValueType::function(
            vec![integer(), integer()],
            ValueType::primitive(PrimitiveType::Text),
            crate::program::expression::FunctionEffect::Pure,
        )
        .unwrap(),
    );
    assert!(error(&program, &value).contains("match its accumulator"));
}

#[test]
fn aggregate_verifier_rejects_corrupt_local_callback_evidence() {
    let (mut program, _) = raw("map([1], fn(value: int) -> int effect pure { value })");
    let value = collection(&program);
    let CoreInstructionKind::Collection { callable, .. } = value.kind else {
        unreachable!()
    };
    let callable = program
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|instruction| instruction.id == callable)
        .unwrap();
    callable.metadata.callable = Some(CallableContract::Bound(FunctionEffect::Local));
    assert!(error(&program, &value).contains("LocalMutation"));
}
