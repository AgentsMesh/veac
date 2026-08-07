use super::super::{
    CallableContract, CoreValueMetadata, EffectEvidence, FunctionSummary, MetadataPath,
    ProjectionStep, Stage,
};
use crate::program::expression::{Effect, FunctionEffect, PrimitiveType, ValueType};
use crate::program::FieldIndex;

#[test]
fn collection_and_map_elements_preserve_nested_callable_contracts() {
    let structure = CoreValueMetadata::structure(&[callable(Effect::Pure)]);
    let list = CoreValueMetadata::list(std::slice::from_ref(&structure));
    assert_callable(list.collection_element().unwrap(), &function_type());

    let mut map = CoreValueMetadata::constant();
    map.append_map_value(&structure);
    let tuple_value = MetadataPath::default().appended(ProjectionStep::TupleElement(1));
    let value = map
        .collection_element()
        .unwrap()
        .project_path(&tuple_value, false)
        .unwrap();
    assert_callable(value, &function_type());
}

#[test]
fn empty_collection_uses_declared_field_effect_as_a_closed_fallback() {
    let formal =
        CoreValueMetadata::typed_parameter(Stage::Const, 0, &PrimitiveType::Integer.into())
            .collection_element()
            .unwrap()
            .struct_field(FieldIndex::new(0), &function_type())
            .unwrap();
    let bound = formal.bind(&[CoreValueMetadata::list(&[])], &[]).unwrap();
    assert_eq!(
        bound.known_callable_effect(&[]).unwrap().summary(),
        Effect::Pure
    );
    assert_eq!(
        bound.callable,
        Some(CallableContract::Bound(FunctionEffect::Pure))
    );
}

#[test]
fn heterogeneous_collection_joins_actual_reachable_effects() {
    let list = CoreValueMetadata::list(&[callable(Effect::Pure), callable(Effect::GraphEmit)]);
    let element = list.collection_element().unwrap();
    assert_eq!(
        element.known_callable_effect(&[]).unwrap().summary(),
        Effect::GraphEmit
    );
}

fn assert_callable(value: CoreValueMetadata, function: &ValueType) {
    let selected = value.struct_field(FieldIndex::new(0), function).unwrap();
    assert!(matches!(
        selected.callable,
        Some(CallableContract::Closure { .. })
    ));
}

fn callable(effect: Effect) -> CoreValueMetadata {
    CoreValueMetadata::closure(
        &FunctionSummary {
            result: CoreValueMetadata::constant(),
            effect: EffectEvidence::from_effect(effect),
            deferred_effects: Vec::new(),
            instruction_count: 1,
        },
        &[],
    )
}

fn function_type() -> ValueType {
    ValueType::function(
        Vec::new(),
        PrimitiveType::Integer.into(),
        FunctionEffect::Pure,
    )
    .unwrap()
}
