use super::super::{CallableContract, CoreValueMetadata, EffectEvidence, FunctionSummary, Stage};
use crate::program::expression::{InputId, PrimitiveType, ValueType};
use crate::program::{FieldIndex, VariantIndex};

#[test]
fn struct_projection_selects_only_the_target_field_axes() {
    let first = CoreValueMetadata::input(Stage::Build, InputId::new(1));
    let second = CoreValueMetadata::input(Stage::Temporal, InputId::new(2));
    let structure = CoreValueMetadata::structure(&[first, second]);
    let projected = structure
        .struct_field(FieldIndex::new(0), &PrimitiveType::Integer.into())
        .unwrap();

    assert_eq!(projected.leaf_stage(), Stage::Build);
    assert_eq!(
        projected.leaf_dependencies().leaf_input_ids(),
        [InputId::new(1)]
    );
    assert!(!projected
        .leaf_dependencies()
        .leaf_input_ids()
        .contains(&InputId::new(2)));
}

#[test]
fn parameter_and_capture_paths_bind_to_the_exact_callable_field() {
    let function = function_type();
    let actual = CoreValueMetadata::structure(&[callable()]);
    let parameter =
        CoreValueMetadata::typed_parameter(Stage::Const, 0, &PrimitiveType::Integer.into())
            .struct_field(FieldIndex::new(0), &function)
            .unwrap()
            .bind(std::slice::from_ref(&actual), &[])
            .unwrap();
    let capture = CoreValueMetadata::capture(Stage::Const, 0, &PrimitiveType::Integer.into())
        .struct_field(FieldIndex::new(0), &function)
        .unwrap()
        .bind(&[], &[actual])
        .unwrap();

    assert!(matches!(
        parameter.callable,
        Some(CallableContract::Closure { .. })
    ));
    assert!(matches!(
        capture.callable,
        Some(CallableContract::Closure { .. })
    ));
}

#[test]
fn parameter_path_dependencies_bind_only_the_selected_field() {
    let actual = CoreValueMetadata::structure(&[
        CoreValueMetadata::input(Stage::Build, InputId::new(3)),
        CoreValueMetadata::input(Stage::Temporal, InputId::new(4)),
    ]);
    let bound = CoreValueMetadata::typed_parameter(Stage::Const, 0, &PrimitiveType::Integer.into())
        .struct_field(FieldIndex::new(0), &PrimitiveType::Integer.into())
        .unwrap()
        .bind(&[actual], &[])
        .unwrap();

    assert_eq!(bound.leaf_stage(), Stage::Build);
    assert_eq!(
        bound.leaf_dependencies().leaf_input_ids(),
        [InputId::new(3)]
    );
}

#[test]
fn enum_paths_distinguish_selected_and_impossible_variant_payloads() {
    let function = function_type();
    let actual = CoreValueMetadata::enumeration(VariantIndex::new(1), &[callable()]);
    let formal =
        CoreValueMetadata::typed_parameter(Stage::Const, 0, &PrimitiveType::Integer.into());
    let selected = formal
        .enum_field(VariantIndex::new(1), FieldIndex::new(0), &function)
        .unwrap()
        .bind(std::slice::from_ref(&actual), &[])
        .unwrap();
    let absent = formal
        .enum_field(VariantIndex::new(0), FieldIndex::new(0), &function)
        .unwrap()
        .bind(&[actual], &[])
        .unwrap();

    assert!(matches!(
        selected.callable,
        Some(CallableContract::Closure { .. })
    ));
    assert_eq!(absent.callable, Some(CallableContract::Impossible));

    let variants = [
        CoreValueMetadata::enumeration(VariantIndex::new(0), &[callable()]),
        CoreValueMetadata::enumeration(VariantIndex::new(1), &[callable()]),
    ];
    let refs = variants.iter().collect::<Vec<_>>();
    let mut alternatives = CoreValueMetadata::combine(refs.iter().copied());
    alternatives.join_contract_from(&refs);
    let impossible = formal
        .enum_field(VariantIndex::new(2), FieldIndex::new(0), &function)
        .unwrap()
        .bind(&[alternatives], &[])
        .unwrap();
    assert_eq!(impossible.callable, Some(CallableContract::Impossible));
}

#[test]
fn opaque_function_projection_fails_closed() {
    assert!(CoreValueMetadata::constant()
        .struct_field(FieldIndex::new(0), &function_type())
        .is_none());
}

fn callable() -> CoreValueMetadata {
    CoreValueMetadata::closure(
        &FunctionSummary {
            result: CoreValueMetadata::constant(),
            effect: EffectEvidence::PURE,
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
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap()
}
