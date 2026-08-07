use super::super::{CoreValueMetadata, DependencyMask, EffectEvidence, ProjectionContract, Stage};
use crate::program::expression::InputId;

fn metadata(
    shape_dependencies: DependencyMask,
    leaf_dependencies: DependencyMask,
) -> CoreValueMetadata {
    CoreValueMetadata {
        effect: EffectEvidence::PURE,
        shape_stage: Stage::Const,
        leaf_stage: Stage::Const,
        shape_dependencies,
        leaf_dependencies,
        callable: None,
        deferred: Vec::new(),
        projection: ProjectionContract::Opaque,
    }
}

#[test]
fn parameter_metadata_labels_shape_and_leaf_origins() {
    let value = CoreValueMetadata::parameter(Stage::Const, 3);
    assert!(value.shape_dependencies().depends_on_parameter_shape(3));
    assert!(!value.shape_dependencies().depends_on_parameter_leaf(3));
    assert!(!value.leaf_dependencies().depends_on_parameter_shape(3));
    assert!(value.leaf_dependencies().depends_on_parameter_leaf(3));
}

#[test]
fn input_metadata_labels_shape_and_leaf_origins() {
    let value = CoreValueMetadata::input(Stage::Build, InputId::new(4));
    assert_eq!(
        value.shape_dependencies().shape_input_ids(),
        [InputId::new(4)]
    );
    assert!(value.shape_dependencies().leaf_input_ids().is_empty());
    assert!(value.leaf_dependencies().shape_input_ids().is_empty());
    assert_eq!(
        value.leaf_dependencies().leaf_input_ids(),
        [InputId::new(4)]
    );
}

#[test]
fn combine_preserves_every_source_and_target_axis() {
    let first = metadata(
        DependencyMask::input_leaf(InputId::new(1)),
        DependencyMask::parameter_shape(0),
    );
    let second = metadata(
        DependencyMask::input_shape(InputId::new(2)),
        DependencyMask::parameter_leaf(1),
    );
    let combined = CoreValueMetadata::combine([&first, &second]);

    assert_eq!(
        combined.shape_dependencies().shape_input_ids(),
        [InputId::new(2)]
    );
    assert_eq!(
        combined.shape_dependencies().leaf_input_ids(),
        [InputId::new(1)]
    );
    assert!(combined.leaf_dependencies().depends_on_parameter_shape(0));
    assert!(combined.leaf_dependencies().depends_on_parameter_leaf(1));
}
