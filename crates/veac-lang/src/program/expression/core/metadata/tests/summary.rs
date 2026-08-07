use super::super::{
    CoreValueMetadata, DependencyMask, EffectEvidence, FunctionSummary, ProjectionContract, Stage,
};
use crate::program::expression::InputId;

fn metadata(
    shape_stage: Stage,
    leaf_stage: Stage,
    shape_dependencies: DependencyMask,
    leaf_dependencies: DependencyMask,
) -> CoreValueMetadata {
    CoreValueMetadata {
        effect: EffectEvidence::PURE,
        shape_stage,
        leaf_stage,
        shape_dependencies,
        leaf_dependencies,
        callable: None,
        deferred: Vec::new(),
        projection: ProjectionContract::Opaque,
    }
}

fn argument() -> CoreValueMetadata {
    metadata(
        Stage::Build,
        Stage::Temporal,
        DependencyMask::input_shape(InputId::new(7)),
        DependencyMask::input_leaf(InputId::new(7)),
    )
}

fn instantiate(
    shape_dependencies: DependencyMask,
    leaf_dependencies: DependencyMask,
) -> CoreValueMetadata {
    FunctionSummary {
        result: metadata(
            Stage::Const,
            Stage::Const,
            shape_dependencies,
            leaf_dependencies,
        ),
        effect: EffectEvidence::PURE,
        deferred_effects: Vec::new(),
        instruction_count: 1,
    }
    .instantiate(&[argument()], &[])
    .unwrap()
}

#[test]
fn identity_preserves_matching_shape_and_leaf_axes() {
    let value = instantiate(
        DependencyMask::parameter_shape(0),
        DependencyMask::parameter_leaf(0),
    );
    assert_eq!(value.shape_stage(), Stage::Build);
    assert_eq!(value.leaf_stage(), Stage::Temporal);
    assert_eq!(
        value.shape_dependencies().shape_input_ids(),
        [InputId::new(7)]
    );
    assert!(value.shape_dependencies().leaf_input_ids().is_empty());
    assert!(value.leaf_dependencies().shape_input_ids().is_empty());
    assert_eq!(
        value.leaf_dependencies().leaf_input_ids(),
        [InputId::new(7)]
    );
}

#[test]
fn fixed_wrap_keeps_shape_constant_while_leaf_remains_dynamic() {
    let value = instantiate(DependencyMask::default(), DependencyMask::parameter_leaf(0));
    assert_eq!(value.shape_stage(), Stage::Const);
    assert_eq!(value.leaf_stage(), Stage::Temporal);
    assert!(value.shape_dependencies().is_empty());
    assert_eq!(
        value.leaf_dependencies().leaf_input_ids(),
        [InputId::new(7)]
    );
}

#[test]
fn shape_to_leaf_flow_ignores_temporal_element_leaves() {
    let value = instantiate(
        DependencyMask::default(),
        DependencyMask::parameter_shape(0),
    );
    assert_eq!(value.shape_stage(), Stage::Const);
    assert_eq!(value.leaf_stage(), Stage::Build);
    assert_eq!(
        value.leaf_dependencies().shape_input_ids(),
        [InputId::new(7)]
    );
    assert!(value.leaf_dependencies().leaf_input_ids().is_empty());
}

#[test]
fn leaf_to_shape_flow_is_represented_for_stage_sink_validation() {
    let value = instantiate(DependencyMask::parameter_leaf(0), DependencyMask::default());
    assert_eq!(value.shape_stage(), Stage::Temporal);
    assert_eq!(value.leaf_stage(), Stage::Const);
    assert_eq!(
        value.shape_dependencies().leaf_input_ids(),
        [InputId::new(7)]
    );
    assert!(value.shape_dependencies().shape_input_ids().is_empty());
}
