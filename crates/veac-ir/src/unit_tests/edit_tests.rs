#[path = "edit_tests/apply_edit_guard_tests.rs"]
mod apply_edit_guard_tests;
#[path = "edit_tests/apply_edit_lifecycle_tests.rs"]
mod apply_edit_lifecycle_tests;
#[path = "edit_tests/audio_color_property_tests.rs"]
mod audio_color_property_tests;
#[path = "edit_tests/changed_object_tests.rs"]
mod changed_object_tests;
#[path = "edit_tests/changed_tree_branch_tests.rs"]
mod changed_tree_branch_tests;
#[path = "edit_tests/composition_edit_tests.rs"]
mod composition_edit_tests;
#[path = "edit_tests/crop_curve_tests.rs"]
mod crop_curve_tests;
#[path = "edit_tests/curve_easing_tests.rs"]
mod curve_easing_tests;
#[path = "edit_tests/curve_split_tests.rs"]
mod curve_split_tests;
#[path = "edit_tests/edit_matte_topology_tests.rs"]
mod edit_matte_topology_tests;
#[path = "edit_tests/edit_sidechain_topology_tests.rs"]
mod edit_sidechain_topology_tests;
#[path = "edit_tests/group_propagation_tests.rs"]
mod group_propagation_tests;
#[path = "edit_tests/group_structure_tests.rs"]
mod group_structure_tests;
#[path = "edit_tests/insert_tests.rs"]
mod insert_tests;
#[path = "edit_tests/invariant_tests.rs"]
mod invariant_tests;
#[path = "edit_tests/keyframe_mutation_tests.rs"]
mod keyframe_mutation_tests;
#[path = "edit_tests/keyframe_upsert_tests.rs"]
mod keyframe_upsert_tests;
#[path = "edit_tests/keying_transition_edit_tests.rs"]
mod keying_transition_edit_tests;
#[path = "edit_tests/linked_edit_tests.rs"]
mod linked_edit_tests;
#[path = "edit_tests/linked_split_branch_tests.rs"]
mod linked_split_branch_tests;
#[path = "edit_tests/linked_split_tests.rs"]
mod linked_split_tests;
#[path = "edit_tests/locked_expanded_tests.rs"]
mod locked_expanded_tests;
#[path = "edit_tests/locked_structure_tests.rs"]
mod locked_structure_tests;
#[path = "edit_tests/mask_keyframe_edit_tests.rs"]
mod mask_keyframe_edit_tests;
#[path = "edit_tests/multicam_structure_tests.rs"]
mod multicam_structure_tests;
#[path = "edit_tests/multicam_tests.rs"]
mod multicam_tests;
#[path = "edit_tests/mutation_tests.rs"]
mod mutation_tests;
#[path = "edit_tests/outcome_tests.rs"]
mod outcome_tests;
#[path = "edit_tests/overwrite_tests.rs"]
mod overwrite_tests;
#[path = "edit_tests/precondition_tests.rs"]
mod precondition_tests;
#[path = "edit_tests/property_branch_tests.rs"]
mod property_branch_tests;
#[path = "edit_tests/property_tests.rs"]
mod property_tests;
#[path = "edit_tests/reverse_timing_tests.rs"]
mod reverse_timing_tests;
#[path = "edit_tests/ripple_tests.rs"]
mod ripple_tests;
#[path = "edit_tests/snap_edge_tests.rs"]
mod snap_edge_tests;
#[path = "edit_tests/snap_tests.rs"]
mod snap_tests;
#[path = "edit_tests/source_time_tests.rs"]
mod source_time_tests;
#[path = "edit_tests/stack_tests.rs"]
mod stack_tests;
#[path = "edit_tests/structure_missing_tests.rs"]
mod structure_missing_tests;
#[path = "edit_tests/structure_tests.rs"]
mod structure_tests;
#[path = "edit_tests/support.rs"]
mod support;
#[path = "edit_tests/template_state_tests.rs"]
mod template_state_tests;
#[path = "edit_tests/text_curve_split_tests.rs"]
mod text_curve_split_tests;
#[path = "edit_tests/text_property_rejection_tests.rs"]
mod text_property_rejection_tests;
#[path = "edit_tests/text_property_tests.rs"]
mod text_property_tests;
#[path = "edit_tests/timing_tests.rs"]
mod timing_tests;
#[path = "edit_tests/transaction_edge_tests.rs"]
mod transaction_edge_tests;
#[path = "edit_tests/transition_edit_topology_tests.rs"]
mod transition_edit_topology_tests;
#[path = "edit_tests/transition_overlap_atomicity_tests.rs"]
mod transition_overlap_atomicity_tests;
#[path = "edit_tests/transition_overwrite_tests.rs"]
mod transition_overwrite_tests;

use crate::{
    test_support::{linked_project, range, sample_project, time},
    *,
};
use support::*;

fn batch(id: &str, project: &ProjectEnvelope, operations: Vec<EditOperation>) -> EditBatch {
    EditBatch {
        operation_id: OperationId::new(id).unwrap(),
        base_revision: project.project.revision,
        atomic: true,
        preconditions: vec![],
        operations,
    }
}

fn generated_clip(id: &str, start: i64) -> Clip {
    let project = sample_project();
    let mut clip = project.project.sequences[0].tracks[1].clips[0].clone();
    clip.id = ItemId::new(id).unwrap();
    clip.record_range = range(start, 60);
    clip
}

fn number_key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    }
}

fn point_key(id: &str, at: i64) -> Keyframe<Point> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value: Point {
            x: Length {
                value: at as f64,
                unit: LengthUnit::Pixels,
            },
            y: Length {
                value: at as f64,
                unit: LengthUnit::Pixels,
            },
        },
        interpolation: Interpolation::Linear,
    }
}

fn vec2_key(id: &str, at: i64, value: f64) -> Keyframe<Vec2> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value: Vec2 { x: value, y: value },
        interpolation: Interpolation::Linear,
    }
}

fn applied(outcome: EditOutcome) -> ProjectEnvelope {
    match outcome {
        EditOutcome::Applied { project, .. } => project,
        other => panic!("expected applied, got {other:?}"),
    }
}

fn assert_rejected(outcome: EditOutcome, code: &str) {
    match outcome {
        EditOutcome::Rejected { diagnostics, .. } => {
            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic.code == code),
                "expected {code}, got {diagnostics:?}"
            );
        }
        other => panic!("expected rejected, got {other:?}"),
    }
}

fn magnetic_project() -> ProjectEnvelope {
    let mut project = sample_project();
    let track = &mut project.project.sequences[0].tracks[0];
    track.clips[0].effects.clear();
    track.clips[0].visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    let mut middle = track.clips[0].clone();
    middle.id = ItemId::new("itm_middle").unwrap();
    middle.record_range = range(600, 600);
    set_linear_start(&mut middle, RationalTime::new(600, 600).unwrap());
    let mut right = middle.clone();
    right.id = ItemId::new("itm_right").unwrap();
    right.record_range = range(1200, 600);
    set_linear_start(&mut right, RationalTime::new(1200, 600).unwrap());
    track.clips.extend([middle, right]);
    project
}
