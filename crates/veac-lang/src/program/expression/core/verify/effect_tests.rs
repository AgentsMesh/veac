use super::*;

#[test]
fn graph_summary_cannot_hide_local_mutation() {
    let local = EffectEvidence::from_effect(Effect::LocalMutation);
    let graph = EffectEvidence::from_effect(Effect::GraphEmit);
    let evidence = local.join(graph);
    assert_eq!(evidence.summary(), Effect::GraphEmit);
    assert!(evidence.contains_local_mutation());
    assert!(!graph.covers(local));
    assert!(evidence.covers(local));
    assert!(evidence.covers(graph));
    assert!(!local.covers(graph));
    for operation in [
        CollectionOperation::Map,
        CollectionOperation::Filter,
        CollectionOperation::Fold,
    ] {
        assert_eq!(
            verify_collection(operation, Some(evidence)),
            Err(CollectionEffectViolation::LocalMutation)
        );
    }
}

#[test]
fn unknown_effects_fail_closed_for_every_collection() {
    assert_eq!(
        verify_collection(CollectionOperation::Map, None),
        Err(CollectionEffectViolation::MapUnverified)
    );
    for operation in [CollectionOperation::Filter, CollectionOperation::Fold] {
        assert_eq!(
            verify_collection(operation, None),
            Err(CollectionEffectViolation::PureUnverified)
        );
    }
}

#[test]
fn pure_is_universal_and_graph_emit_is_map_only() {
    let pure = Some(EffectEvidence::PURE);
    for operation in [
        CollectionOperation::Map,
        CollectionOperation::Filter,
        CollectionOperation::Fold,
    ] {
        assert_eq!(verify_collection(operation, pure), Ok(()));
    }
    let graph = Some(EffectEvidence::from_effect(Effect::GraphEmit));
    assert_eq!(verify_collection(CollectionOperation::Map, graph), Ok(()));
    assert_eq!(
        verify_collection(CollectionOperation::Filter, graph),
        Err(CollectionEffectViolation::RequiresPure)
    );
}
