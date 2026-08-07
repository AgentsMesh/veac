use sha2::{Digest, Sha256};

use super::super::instruction;
use crate::program::expression::{
    compile_expression, BlockId, ClosureDefinitionId, CoreForEach, CoreForEachSlotId,
    CoreTerminator, CoreTypeId, Effect, ExpressionContext, TypeEnvironment, ValueId,
};

fn authored() -> CoreForEach {
    let compiled = compile_expression(
        "{ let offset = 1; for value in [1, 2] { value + offset } }",
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap();
    compiled
        .core()
        .blocks()
        .iter()
        .find_map(|block| match block.terminator() {
            CoreTerminator::ForEach(value) => Some(value.clone()),
            _ => None,
        })
        .unwrap()
}

fn hash(value: CoreForEach) -> [u8; 32] {
    let mut digest = Sha256::new();
    instruction::terminator(&mut digest, &CoreTerminator::ForEach(value));
    digest.finalize().into()
}

#[test]
fn for_each_digest_covers_control_slots_captures_provenance_and_effect_evidence() {
    let base = authored();
    let baseline = hash(base.clone());
    let mut variants = Vec::new();
    variants.push(changed(&base, |value| value.iterable = ValueId::new(99)));
    variants.push(changed(&base, |value| value.maximum_count -= 1));
    variants.push(changed(&base, |value| {
        value.element.id = CoreForEachSlotId::Index
    }));
    variants.push(changed(&base, |value| {
        value.element.type_id = CoreTypeId::new(99)
    }));
    variants.push(changed(&base, |value| {
        value.index.id = CoreForEachSlotId::Element
    }));
    variants.push(changed(&base, |value| {
        value.index.type_id = CoreTypeId::new(98)
    }));
    variants.push(changed(&base, |value| {
        value.body = ClosureDefinitionId::new(9)
    }));
    variants.push(changed(&base, |value| {
        value.captures.push(ValueId::new(91))
    }));
    variants.push(changed(&base, |value| {
        value.continuation = BlockId::new(92)
    }));
    variants.push(changed(&base, |value| {
        value.provenance.definition = ClosureDefinitionId::new(8)
    }));
    variants.push(changed(&base, |value| value.provenance.loop_span.end += 1));
    variants.push(changed(&base, |value| {
        value.provenance.binding_span.end += 1
    }));
    variants.push(changed(&base, |value| {
        value.effect.summary = Effect::GraphEmit
    }));
    variants.push(changed(&base, |value| {
        value.effect.contains_local_mutation = true
    }));
    for variant in variants {
        assert_ne!(hash(variant), baseline);
    }
}

fn changed(base: &CoreForEach, mutate: impl FnOnce(&mut CoreForEach)) -> CoreForEach {
    let mut value = base.clone();
    mutate(&mut value);
    value
}
