use super::{raw, verify_error};
use crate::program::expression::core::closure_digest;
use crate::program::expression::{
    ClosureDefinitionId, CoreForEach, CoreForEachSlotId, CoreInstructionKind, CoreProgram,
    CoreTerminator, Effect,
};

fn loop_value(program: &CoreProgram, ordinal: usize) -> &CoreForEach {
    program
        .blocks
        .iter()
        .filter_map(|block| match &block.terminator {
            CoreTerminator::ForEach(value) => Some(value),
            _ => None,
        })
        .nth(ordinal)
        .unwrap()
}

fn loop_value_mut(program: &mut CoreProgram, ordinal: usize) -> &mut CoreForEach {
    program
        .blocks
        .iter_mut()
        .filter_map(|block| match &mut block.terminator {
            CoreTerminator::ForEach(value) => Some(value),
            _ => None,
        })
        .nth(ordinal)
        .unwrap()
}

fn failure(program: CoreProgram, functions: &crate::program::expression::FunctionMap) -> String {
    verify_error(program, functions).message().to_owned()
}

#[test]
fn for_each_rejects_invalid_bounds_body_and_provenance() {
    let (base, functions) = raw("for value in [1, 2] { value }");
    let mut zero = base.clone();
    loop_value_mut(&mut zero, 0).maximum_count = 0;
    assert!(failure(zero, &functions).contains("maximum count"));
    let mut overflow = base.clone();
    loop_value_mut(&mut overflow, 0).maximum_count = u32::MAX;
    assert!(failure(overflow, &functions).contains("maximum count"));

    let mut body = base.clone();
    let missing = ClosureDefinitionId::new(99);
    loop_value_mut(&mut body, 0).body = missing;
    loop_value_mut(&mut body, 0).provenance.definition = missing;
    assert!(failure(body, &functions).contains("body definition is unavailable"));
    let mut provenance = base.clone();
    loop_value_mut(&mut provenance, 0).provenance.definition = missing;
    assert!(failure(provenance, &functions).contains("provenance"));
    let mut binding = base;
    loop_value_mut(&mut binding, 0).provenance.binding_span = 0..0;
    assert!(failure(binding, &functions).contains("binding span"));
}

#[test]
fn for_each_rejects_forged_slots_captures_and_result_contracts() {
    let source = "{ let offset = 1; for value in [1, 2] { value + offset } }";
    let (base, functions) = raw(source);
    let mut element = base.clone();
    loop_value_mut(&mut element, 0).element.id = CoreForEachSlotId::Index;
    assert!(failure(element, &functions).contains("activation slots"));
    let mut index = base.clone();
    loop_value_mut(&mut index, 0).index.id = CoreForEachSlotId::Element;
    assert!(failure(index, &functions).contains("activation slots"));

    let mut arity = base.clone();
    loop_value_mut(&mut arity, 0).captures.clear();
    assert!(failure(arity, &functions).contains("capture arity"));
    let mut capture_type = base.clone();
    let iterable = loop_value(&capture_type, 0).iterable;
    loop_value_mut(&mut capture_type, 0).captures[0] = iterable;
    assert!(failure(capture_type, &functions).contains("capture type"));

    let mut result_type = base.clone();
    let continuation = loop_value(&result_type, 0).continuation.index().unwrap();
    let scalar_type = loop_value(&result_type, 0).element.type_id;
    result_type.blocks[continuation].parameters[0].type_id = scalar_type;
    assert!(failure(result_type, &functions).contains("result type"));
    let mut evidence = base.clone();
    loop_value_mut(&mut evidence, 0).effect.summary = Effect::GraphEmit;
    assert!(failure(evidence, &functions).contains("effect evidence"));
    let mut metadata = base;
    loop_value_mut(&mut metadata, 0).result_metadata.leaf_stage =
        crate::program::expression::Stage::Temporal;
    assert!(failure(metadata, &functions).contains("result or effect evidence"));
}

#[test]
fn for_each_body_cannot_be_reused_or_materialized_as_a_closure() {
    let source = "{ let first = for value in [1] { value }; \
                  let second = for value in [2] { value }; second }";
    let (mut reused, functions) = raw(source);
    let first = loop_value(&reused, 0).body;
    let first_provenance = loop_value(&reused, 0).provenance.clone();
    let second = loop_value(&reused, 1).body.index().unwrap();
    loop_value_mut(&mut reused, 1).body = first;
    loop_value_mut(&mut reused, 1).provenance = first_provenance;
    let definition = &mut reused.closure_definitions[second];
    definition.non_escaping = false;
    definition.digest = closure_digest(
        &definition.parameter_types,
        &definition.parameter_stages,
        &definition.capture_types,
        definition.effect,
        false,
        &definition.body,
    );
    let message = failure(reused, &functions);
    assert!(
        message.contains("exactly one Collection or for-each owner"),
        "{message}"
    );

    let source = "{ let callback = fn(value: int, index: int) -> int effect emit { value }; \
                  let values = for value in [1] { value }; values }";
    let (mut materialized, functions) = raw(source);
    let body = loop_value(&materialized, 0).body;
    let closure = materialized
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Closure { .. }))
        .unwrap();
    let CoreInstructionKind::Closure { definition, .. } = &mut closure.kind else {
        unreachable!()
    };
    *definition = body;
    let message = failure(materialized, &functions);
    assert!(
        message.contains("exactly one Collection or for-each owner"),
        "{message}"
    );
}
