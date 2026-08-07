use crate::{
    test_support::{sample_project, time},
    *,
};

use super::support::{add_progress_clock, bind, binding};

fn bound_project() -> ProjectEnvelope {
    let mut project = sample_project();
    let id = bind(&mut project, TemporalType::Scalar, "edited");
    project.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = binding(&id);
    add_progress_clock(&mut project, "itm_video");
    project
}

fn batch(project: &ProjectEnvelope, operation: EditOperation, id: &str) -> EditBatch {
    EditBatch {
        operation_id: OperationId::new(id).unwrap(),
        base_revision: project.project.revision,
        atomic: true,
        preconditions: Vec::new(),
        operations: vec![operation],
    }
}

fn rejection_codes(outcome: EditOutcome) -> Vec<String> {
    match outcome {
        EditOutcome::Rejected { diagnostics, .. } => {
            diagnostics.into_iter().map(|value| value.code).collect()
        }
        value => panic!("expected rejection, got {value:?}"),
    }
}

#[test]
fn keyframe_upsert_cannot_silently_replace_a_temporal_binding() {
    let project = bound_project();
    let operation = EditOperation::EditKeyframe {
        edit: KeyframeEdit::UpsertNumber {
            clip_id: ItemId::new("itm_video").unwrap(),
            target: NumberCurveTarget::VisualOpacity,
            keyframe: Keyframe {
                id: KeyframeId::new("kf_bound").unwrap(),
                time: time(0),
                value: 0.5,
                interpolation: Interpolation::Linear,
            },
        },
    };
    let outcome = apply_edit_batch(&project, &batch(&project, operation, "op_bound_key"));
    let codes = rejection_codes(outcome);
    assert_eq!(codes, ["EDIT_REJECTED"]);
}

#[test]
fn trim_preserves_an_owner_bound_program_and_remains_valid() {
    let project = bound_project();
    let operation = EditOperation::TrimClip {
        clip_id: ItemId::new("itm_video").unwrap(),
        edge: TrimEdge::Out,
        delta: RationalTime::new(-60, 600).unwrap(),
        ripple: false,
    };
    let outcome = apply_edit_batch(&project, &batch(&project, operation, "op_bound_trim"));
    let updated = match outcome {
        EditOutcome::Applied { project, .. } => project,
        value => panic!("expected applied trim, got {value:?}"),
    };
    let opacity = &updated.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .opacity;
    assert_eq!(opacity.binding_id().unwrap().as_str(), "tbd_edited");
    validate(&updated).unwrap();
}

#[test]
fn split_cannot_duplicate_an_item_owned_binding_onto_a_new_item() {
    let project = bound_project();
    let operation = EditOperation::SplitClip {
        clip_id: ItemId::new("itm_video").unwrap(),
        at: time(300),
        right_clip_id: ItemId::new("itm_video_right").unwrap(),
        relation_fragments: Vec::new(),
    };
    let outcome = apply_edit_batch(&project, &batch(&project, operation, "op_bound_split"));
    assert!(rejection_codes(outcome).contains(&"TEMPORAL_SINK_CLOCK_OWNER".to_owned()));
    assert_eq!(project.project.sequences[0].tracks[0].clips.len(), 1);
}

#[test]
fn highlight_keyframes_participate_in_targeting_lookup_and_curve_slicing() {
    let project = sample_project();
    let clip_id = ItemId::new("itm_caption").unwrap();
    let key = |id: &str, at: i64, value: f64| Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    };
    let animation = TextAnimation {
        granularity: TextGranularity::Word,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: Some(TextHighlightAnimation {
            fill: Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            progress: Animatable::Keyframes {
                keyframes: vec![
                    key("kf_highlight_start", 0, 0.0),
                    key("kf_highlight_end", 300, 1.0),
                ],
            },
        }),
        opacity: Animatable::constant(1.0),
        stagger: time(0),
    };
    let operations = vec![
        EditOperation::SetTextProperty {
            clip_id: clip_id.clone(),
            property: TextProperty::Animation(Some(animation)),
        },
        EditOperation::EditKeyframe {
            edit: KeyframeEdit::UpsertNumber {
                clip_id: clip_id.clone(),
                target: NumberCurveTarget::TextHighlightProgress,
                keyframe: key("kf_highlight_mid", 150, 0.5),
            },
        },
        EditOperation::EditKeyframe {
            edit: KeyframeEdit::Move {
                clip_id: clip_id.clone(),
                keyframe_id: KeyframeId::new("kf_highlight_mid").unwrap(),
                time: time(180),
            },
        },
        EditOperation::EditKeyframe {
            edit: KeyframeEdit::Remove {
                clip_id: clip_id.clone(),
                keyframe_id: KeyframeId::new("kf_highlight_start").unwrap(),
            },
        },
        EditOperation::TrimClip {
            clip_id,
            edge: TrimEdge::Out,
            delta: time(-60),
            ripple: false,
        },
    ];
    let request = EditBatch {
        operation_id: OperationId::new("op_highlight_keyframes").unwrap(),
        base_revision: project.project.revision,
        atomic: true,
        preconditions: Vec::new(),
        operations,
    };
    let outcome = apply_edit_batch(&project, &request);
    assert!(matches!(outcome, EditOutcome::Applied { .. }));
}
