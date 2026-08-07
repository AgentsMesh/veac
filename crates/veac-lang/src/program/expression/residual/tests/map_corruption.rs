use super::support::*;
use crate::program::expression::{CoreInstructionKind, CoreTypeId, ValueId};
use veac_ir::TemporalType;

#[test]
fn map_keys_reject_wrong_types_ordinals_and_duplicates() {
    let mut wrong_type = compile("#{\"a\": 1}", &[]);
    let (key, value) = map_operands(&wrong_type);
    set_first_key(&mut wrong_type, value, 0);
    assert_eq!(
        failure(&wrong_type, "map_key_type"),
        "RESIDUAL_CORE_CONTRACT"
    );

    let mut ordinal = compile("#{\"a\": 1}", &[]);
    set_first_key(&mut ordinal, key, 1);
    assert_eq!(failure(&ordinal, "map_ordinal"), "RESIDUAL_CORE_CONTRACT");

    let mut duplicate = compile("#{\"a\": 1, \"b\": 2}", &[]);
    let keys = duplicate.core().blocks()[0]
        .instructions()
        .iter()
        .filter_map(|instruction| match instruction.kind() {
            CoreInstructionKind::MapKey { key, .. } => Some(*key),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut seen = 0;
    for instruction in &mut duplicate.program.core_mut().blocks[0].instructions {
        if let CoreInstructionKind::MapKey { key, .. } = &mut instruction.kind {
            if seen == 1 {
                *key = keys[0];
            }
            seen += 1;
        }
    }
    assert_eq!(
        failure(&duplicate, "map_duplicate"),
        "EXPRESSION_DUPLICATE_MAP_KEY"
    );
}

#[test]
fn map_values_and_builder_contracts_fail_closed_after_verification() {
    let mut wrong_value = compile("#{\"a\": 1}", &[]);
    let (key, _) = map_operands(&wrong_value);
    for instruction in &mut wrong_value.program.core_mut().blocks[0].instructions {
        if let CoreInstructionKind::MapValue { value, .. } = &mut instruction.kind {
            *value = key;
        }
    }
    assert_eq!(
        failure(&wrong_value, "map_value_type"),
        "RESIDUAL_CORE_CONTRACT"
    );

    let mut count = compile("#{\"a\": 1}", &[]);
    for instruction in &mut count.program.core_mut().blocks[0].instructions {
        if let CoreInstructionKind::MapBegin { entries } = &mut instruction.kind {
            *entries = 2;
        }
    }
    assert_eq!(failure(&count, "map_count"), "RESIDUAL_CORE_CONTRACT");

    let mut map_type = compile("#{\"a\": 1}", &[]);
    let begin = map_type.program.core_mut().blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::MapBegin { .. }))
        .unwrap();
    begin.type_id = CoreTypeId::new(u32::MAX);
    assert_eq!(failure(&map_type, "map_type"), "RESIDUAL_CORE_CONTRACT");
}

#[test]
fn temporal_map_entries_are_rejected_after_a_corrupted_verified_core() {
    let input = [("clock", parameter("map_clock", TemporalType::Scalar))];
    let mut expression = compile(
        "{ let clock_first = clock; let value = #{\"a\": 1}; clock_first }",
        &input,
    );
    let input_id = expression.core().blocks()[0]
        .instructions()
        .iter()
        .find(|value| matches!(value.kind(), CoreInstructionKind::Input(_)))
        .unwrap()
        .id();
    set_first_key(&mut expression, input_id, 0);
    assert_eq!(
        failure(&expression, "map_temporal"),
        "RESIDUAL_VALUE_UNSUPPORTED"
    );
    let result = residualize(&compile("#{\"a\": 1}", &[]), &no_bindings(), "map_ok");
    assert!(matches!(
        result.value(),
        crate::program::expression::ResidualRuntimeValue::Concrete(_)
    ));
    assert!(result.program().is_none());
}

fn map_operands(expression: &crate::program::expression::CompiledExpression) -> (ValueId, ValueId) {
    let block = &expression.core().blocks()[0];
    let key = block
        .instructions()
        .iter()
        .find_map(|instruction| match instruction.kind() {
            CoreInstructionKind::MapKey { key, .. } => Some(*key),
            _ => None,
        })
        .unwrap();
    let value = block
        .instructions()
        .iter()
        .find_map(|instruction| match instruction.kind() {
            CoreInstructionKind::MapValue { value, .. } => Some(*value),
            _ => None,
        })
        .unwrap();
    (key, value)
}

fn set_first_key(
    expression: &mut crate::program::expression::CompiledExpression,
    value: ValueId,
    ordinal: u32,
) {
    for instruction in &mut expression.program.core_mut().blocks[0].instructions {
        if let CoreInstructionKind::MapKey {
            key,
            ordinal: target,
            ..
        } = &mut instruction.kind
        {
            *key = value;
            *target = ordinal;
            return;
        }
    }
}

fn failure(
    expression: &crate::program::expression::CompiledExpression,
    name: &str,
) -> &'static str {
    super::super::residualize_expression(expression, &no_bindings(), request(name))
        .unwrap_err()
        .code()
}
