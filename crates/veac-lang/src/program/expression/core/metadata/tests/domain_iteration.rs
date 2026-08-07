use super::super::{CoreValueMetadata, Effect, Stage};
use crate::program::{DomainOperationId, DomainOperationRegistry};

#[test]
fn temporal_entity_keys_are_visible_to_the_topology_stage_guard() {
    let registry = DomainOperationRegistry::standard();
    let contract = registry.lookup(DomainOperationId::Sequence).unwrap();
    let key = CoreValueMetadata::pure(Stage::Temporal, Default::default(), Default::default());
    let emitted = CoreValueMetadata::domain(contract, [&key].into_iter());
    assert_eq!(emitted.effect(), Effect::GraphEmit);
    assert_eq!(emitted.shape_stage(), Stage::Temporal);
}

#[test]
fn temporal_map_leaves_become_temporal_at_a_topology_attachment_sink() {
    let registry = DomainOperationRegistry::standard();
    let contract = registry
        .lookup(DomainOperationId::ProjectWithSequences)
        .unwrap();
    let project = CoreValueMetadata::pure(Stage::Build, Default::default(), Default::default());
    let temporal = CoreValueMetadata::pure(Stage::Temporal, Default::default(), Default::default());
    let mut mapped = CoreValueMetadata::constant();
    mapped.absorb_leaf(&temporal);

    assert_eq!(mapped.shape_stage(), Stage::Const);
    assert_eq!(mapped.leaf_stage(), Stage::Temporal);
    let attached = CoreValueMetadata::domain(contract, [&project, &mapped].into_iter());
    assert_eq!(attached.shape_stage(), Stage::Temporal);
}

#[test]
fn resource_path_and_reference_are_topology_dependencies() {
    let registry = DomainOperationRegistry::standard();
    let constant = CoreValueMetadata::constant();
    let temporal = CoreValueMetadata::pure(Stage::Temporal, Default::default(), Default::default());
    let contract = registry.lookup(DomainOperationId::ImageResource).unwrap();
    let path_temporal =
        CoreValueMetadata::domain(contract, [&constant, &temporal, &constant].into_iter());
    let identity_temporal =
        CoreValueMetadata::domain(contract, [&constant, &constant, &temporal].into_iter());
    for image in [&path_temporal, &identity_temporal] {
        assert_eq!(image.effect(), Effect::GraphEmit);
        assert_eq!(image.shape_stage(), Stage::Temporal);
    }

    let media = CoreValueMetadata::domain(
        registry.lookup(DomainOperationId::Media).unwrap(),
        [&identity_temporal].into_iter(),
    );
    assert_eq!(media.effect(), Effect::GraphEmit);
    assert_eq!(media.shape_stage(), Stage::Temporal);
    assert_eq!(media.leaf_stage(), Stage::Const);
}

#[test]
fn temporal_caption_content_remains_a_leaf_dependency() {
    let registry = DomainOperationRegistry::standard();
    let constant = CoreValueMetadata::constant();
    let build = CoreValueMetadata::pure(Stage::Build, Default::default(), Default::default());
    let temporal = CoreValueMetadata::pure(Stage::Temporal, Default::default(), Default::default());
    let caption = CoreValueMetadata::domain(
        registry.lookup(DomainOperationId::CaptionItem).unwrap(),
        [&constant, &temporal, &build, &build].into_iter(),
    );
    assert_eq!(caption.effect(), Effect::GraphEmit);
    assert_eq!(caption.shape_stage(), Stage::Build);
    assert_eq!(caption.leaf_stage(), Stage::Temporal);
}
