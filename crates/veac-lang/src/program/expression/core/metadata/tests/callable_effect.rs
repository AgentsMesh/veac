use std::sync::Arc;

use super::super::{
    BindingRoot, CallableContract, CoreValueMetadata, DeferredCall, Effect, EffectEvidence,
    FunctionSummary, MetadataPath, Stage,
};
use crate::program::expression::{FunctionEffect, PrimitiveType, ValueType, MAX_EXPRESSION_DEPTH};

#[test]
fn captured_callable_effects_resolve_recursively() {
    let emitted = closure(Effect::GraphEmit, Vec::new(), &[]);
    let middle = closure(
        Effect::Pure,
        vec![deferred(BindingRoot::Capture(0))],
        std::slice::from_ref(&emitted),
    );
    let outer = closure(
        Effect::Pure,
        vec![deferred(BindingRoot::Capture(0))],
        &[middle],
    );

    assert_eq!(known_summary(&outer), Some(Effect::GraphEmit));
}

#[test]
fn invocation_arguments_close_higher_order_effects() {
    let apply = closure(Effect::Pure, vec![deferred(BindingRoot::Parameter(0))], &[]);
    let pure = closure(Effect::Pure, Vec::new(), &[]);
    let emitted = closure(Effect::GraphEmit, Vec::new(), &[]);

    assert_eq!(known_summary_with(&apply, &[pure]), Some(Effect::Pure));
    assert_eq!(
        known_summary_with(&apply, &[emitted]),
        Some(Effect::GraphEmit)
    );
    assert_eq!(
        known_summary_with(&apply, &[CoreValueMetadata::constant()]),
        Some(Effect::Pure)
    );
}

#[test]
fn joins_require_every_reachable_effect_to_be_known() {
    let pure = closure(Effect::Pure, Vec::new(), &[]);
    let emitted = closure(Effect::GraphEmit, Vec::new(), &[]);
    let pure_bound = CoreValueMetadata::typed_parameter(Stage::Const, 0, &function_type());
    let unknown = with_contract(CallableContract::Bound(FunctionEffect::Any));
    let known = joined(&[&pure, &emitted]);
    let bounded = joined(&[&pure, &pure_bound]);
    let partial = joined(&[&pure, &unknown]);
    let impossible = with_contract(CallableContract::Impossible);

    assert_eq!(known_summary(&known), Some(Effect::GraphEmit));
    assert_eq!(known_summary(&bounded), Some(Effect::Pure));
    assert_eq!(partial.known_callable_effect(&[]), None);
    assert_eq!(known_summary(&impossible), Some(Effect::Pure));
}

#[test]
fn direct_graph_summary_retains_local_mutation_evidence() {
    let effects = EffectEvidence::from_effect(Effect::LocalMutation)
        .join(EffectEvidence::from_effect(Effect::GraphEmit));
    let summary = function_summary(effects, Vec::new());
    assert_eq!(summary.effect(), Effect::GraphEmit);
    assert!(summary.contains_local_mutation());
    let callback = CoreValueMetadata::closure(&summary, &[]);
    let resolved = callback.known_callable_effect(&[]).unwrap();
    let invoked =
        CoreValueMetadata::invoke(&callback, &[], &PrimitiveType::Integer.into()).unwrap();

    assert_eq!(resolved.summary(), Effect::GraphEmit);
    assert!(resolved.contains_local_mutation());
    assert_eq!(invoked.effect(), Effect::GraphEmit);
    assert!(invoked.contains_local_mutation());
}

#[test]
fn captured_effects_resolve_recursively_without_masking_mutation() {
    let local = closure(Effect::LocalMutation, Vec::new(), &[]);
    let emitted = closure(Effect::GraphEmit, Vec::new(), &[]);
    let middle = closure(
        Effect::Pure,
        vec![
            deferred(BindingRoot::Capture(0)),
            deferred(BindingRoot::Capture(1)),
        ],
        &[local, emitted],
    );
    let outer = closure(
        Effect::Pure,
        vec![deferred(BindingRoot::Capture(0))],
        &[middle],
    );
    let resolved = outer.known_callable_effect(&[]).unwrap();

    assert_eq!(resolved.summary(), Effect::GraphEmit);
    assert!(resolved.contains_local_mutation());
}

#[test]
fn recursive_effect_resolution_exhaustion_fails_closed() {
    let mut value = closure(Effect::Pure, Vec::new(), &[]);
    for _ in 0..=MAX_EXPRESSION_DEPTH {
        value = closure(
            Effect::Pure,
            vec![deferred(BindingRoot::Capture(0))],
            &[value],
        );
    }
    assert_eq!(value.known_callable_effect(&[]), None);
}

fn closure(
    effect: Effect,
    deferred_effects: Vec<Arc<DeferredCall>>,
    captures: &[CoreValueMetadata],
) -> CoreValueMetadata {
    closure_with_evidence(
        EffectEvidence::from_effect(effect),
        deferred_effects,
        captures,
    )
}

fn closure_with_evidence(
    effect: EffectEvidence,
    deferred_effects: Vec<Arc<DeferredCall>>,
    captures: &[CoreValueMetadata],
) -> CoreValueMetadata {
    CoreValueMetadata::closure(&function_summary(effect, deferred_effects), captures)
}

fn function_summary(
    effect: EffectEvidence,
    deferred_effects: Vec<Arc<DeferredCall>>,
) -> FunctionSummary {
    FunctionSummary {
        result: CoreValueMetadata::constant(),
        effect,
        deferred_effects,
        instruction_count: 1,
    }
}

fn known_summary(value: &CoreValueMetadata) -> Option<Effect> {
    known_summary_with(value, &[])
}

fn known_summary_with(
    value: &CoreValueMetadata,
    arguments: &[CoreValueMetadata],
) -> Option<Effect> {
    value
        .known_callable_effect(arguments)
        .map(EffectEvidence::summary)
}

fn deferred(root: BindingRoot) -> Arc<DeferredCall> {
    Arc::new(DeferredCall {
        callee: CallableContract::Binding {
            root,
            path: MetadataPath::default(),
            fallback: crate::program::expression::FunctionEffect::Pure,
        },
        arguments: Vec::new().into(),
        result_type: PrimitiveType::Integer.into(),
    })
}

fn joined(values: &[&CoreValueMetadata]) -> CoreValueMetadata {
    with_contract(CoreValueMetadata::join_function(values).unwrap())
}

fn with_contract(contract: CallableContract) -> CoreValueMetadata {
    let mut value = CoreValueMetadata::constant();
    value.callable = Some(contract);
    value
}

fn function_type() -> ValueType {
    ValueType::function(
        Vec::new(),
        PrimitiveType::Integer.into(),
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap()
}
