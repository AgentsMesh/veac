use std::sync::Arc;

use super::super::{
    CallableContract, CoreValueMetadata, DeferredCall, DeferredUse, EffectEvidence,
    FunctionSummary, MetadataPath,
};
use crate::program::expression::{CollectionOperation, Effect, PrimitiveType, ValueType};

#[test]
fn aggregate_axes_preserve_deferred_shape_and_leaf_uses() {
    let mut iterable = CoreValueMetadata::constant();
    iterable.deferred.push(deferred());
    let callback = pure_callback();
    let initial = CoreValueMetadata::constant();
    let integer: ValueType = PrimitiveType::Integer.into();
    let boolean: ValueType = PrimitiveType::Boolean.into();
    let list = ValueType::list(integer.clone()).unwrap();

    let mapped = CoreValueMetadata::aggregate(
        CollectionOperation::Map,
        &iterable,
        None,
        &callback,
        &integer,
        &list,
    )
    .unwrap();
    assert!(mapped.deferred.iter().any(|value| value.shape));

    let filtered = CoreValueMetadata::aggregate(
        CollectionOperation::Filter,
        &iterable,
        None,
        &callback,
        &boolean,
        &list,
    )
    .unwrap();
    assert!(filtered.deferred.iter().any(|value| value.leaf));

    let folded = CoreValueMetadata::aggregate(
        CollectionOperation::Fold,
        &iterable,
        Some(&initial),
        &callback,
        &integer,
        &integer,
    )
    .unwrap();
    assert_eq!(folded.effect(), Effect::Pure);
    assert!(CoreValueMetadata::aggregate(
        CollectionOperation::Fold,
        &iterable,
        None,
        &callback,
        &integer,
        &integer,
    )
    .is_none());
}

fn pure_callback() -> CoreValueMetadata {
    let summary = FunctionSummary {
        result: CoreValueMetadata::constant(),
        effect: EffectEvidence::PURE,
        deferred_effects: Vec::new(),
        instruction_count: 1,
    };
    CoreValueMetadata::closure(&summary, &[])
}

fn deferred() -> DeferredUse {
    DeferredUse {
        call: Arc::new(DeferredCall {
            callee: CallableContract::Impossible,
            arguments: Vec::new().into(),
            result_type: PrimitiveType::Integer.into(),
        }),
        path: MetadataPath::default(),
        shape: true,
        leaf: true,
    }
}
