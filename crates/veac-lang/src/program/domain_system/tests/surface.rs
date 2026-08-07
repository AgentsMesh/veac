use std::collections::BTreeSet;

use super::*;

#[test]
fn every_v7_contract_has_one_resolvable_surface_exposure() {
    let registry = DomainOperationRegistry::standard();
    let mut resolved = BTreeSet::new();
    for contract in registry.contracts() {
        let actual = match contract.exposure() {
            DomainOperationExposure::FreeFunction { name } => registry.lookup_function(name),
            DomainOperationExposure::Method { receiver, name } => {
                registry.lookup_method(*receiver, name)
            }
        };
        assert_eq!(actual, Some(contract));
        assert!(resolved.insert(contract.id()));
    }
    assert_eq!(resolved.len(), 581);
}

#[test]
fn representative_free_functions_and_owner_methods_are_typed() {
    let registry = DomainOperationRegistry::standard();
    for (name, id) in [
        ("canvas", DomainOperationId::Canvas),
        ("project_settings", DomainOperationId::ProjectSettings),
        ("source_media", DomainOperationId::SourceMedia),
        ("visual_style", DomainOperationId::VisualStyle),
        ("audio_style", DomainOperationId::AudioStyle),
        ("delivery", DomainOperationId::Delivery),
    ] {
        assert_eq!(
            registry.lookup_function(name).map(|value| value.id()),
            Some(id)
        );
    }
    for (receiver, name, id) in [
        (
            DomainType::Project,
            "with_resource",
            DomainOperationId::ProjectWithResource,
        ),
        (
            DomainType::Sequence,
            "with_relation",
            DomainOperationId::SequenceWithRelation,
        ),
        (
            DomainType::Layer,
            "with_item",
            DomainOperationId::LayerWithItem,
        ),
        (
            DomainType::Item,
            "with_effect",
            DomainOperationId::ItemWithEffect,
        ),
    ] {
        assert_eq!(
            registry
                .lookup_method(receiver, name)
                .map(|value| value.id()),
            Some(id)
        );
    }
}

#[test]
fn internal_names_and_wrong_receivers_fail_closed() {
    let registry = DomainOperationRegistry::standard();
    assert!(registry.lookup_function("project_with_sequence").is_none());
    assert!(registry.lookup_function("with_sequence").is_none());
    assert!(registry
        .lookup_method(DomainType::Sequence, "with_resource")
        .is_none());
    assert!(registry.lookup_function("nonexistent").is_none());
}
