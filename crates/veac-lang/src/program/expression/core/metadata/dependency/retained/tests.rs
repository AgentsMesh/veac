use std::mem::size_of;

use super::*;
use crate::program::expression::core::metadata::{BindingRoot, MetadataPath};
use crate::program::FieldIndex;

#[test]
fn retained_payload_counts_binding_elements_and_path_steps_exactly() {
    let path = MetadataPath::default()
        .appended(ProjectionStep::StructField(FieldIndex::new(0)))
        .appended(ProjectionStep::StructField(FieldIndex::new(1)));
    let mut mask = DependencyMask::binding_shape(BindingRoot::Parameter(0), path);
    mask.union(&DependencyMask::binding_leaf(
        BindingRoot::Capture(0),
        MetadataPath::default(),
    ));
    mask.union(&DependencyMask::input_shape(InputId::new(0)));

    assert_eq!(
        mask.retained_payload_bytes(),
        Some(
            size_of::<InputId>()
                + 2 * size_of::<BindingDependency>()
                + 2 * size_of::<ProjectionStep>()
        )
    );
}

#[test]
fn retained_payload_arithmetic_fails_closed_on_overflow() {
    assert_eq!(dependency_payload_bytes(usize::MAX, 0, 0), None);
    assert_eq!(dependency_payload_bytes(0, usize::MAX, 0), None);
    assert_eq!(dependency_payload_bytes(0, 0, usize::MAX), None);
    assert_eq!(checked_sum([usize::MAX, 1]), None);
}
