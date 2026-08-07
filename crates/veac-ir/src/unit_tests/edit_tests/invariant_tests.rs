use crate::test_support::time;

use super::*;

#[test]
fn final_invariant_failure_rolls_back_the_batch() {
    let project = sample_project();
    let edit = batch(
        "op_rollback",
        &project,
        vec![
            EditOperation::SetClipEnabled {
                clip_id: ItemId::new("itm_caption").unwrap(),
                enabled: false,
            },
            EditOperation::SetText {
                clip_id: ItemId::new("itm_caption").unwrap(),
                text: String::new(),
            },
        ],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "EMPTY_TEXT");
    assert!(project.project.sequences[0].tracks[1].clips[0].enabled);

    let invalid_time = batch(
        "op_invalid_time",
        &project,
        vec![EditOperation::MoveClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
            record_start: RationalTime::new(-1, 600).unwrap(),
        }],
    );
    assert_rejected(apply_edit_batch(&project, &invalid_time), "RECORD_RANGE");
}

#[test]
fn invalid_values_and_revision_overflow_are_rejected() {
    let project = sample_project();
    let source = ClipSource::Caption {
        text: "invalid".to_owned(),
        speaker: None,
        cue: Box::default(),
        style: TextStyle {
            font: FontRef::Family {
                family: "Inter".to_owned(),
            },
            size_pixels: f64::NAN,
            color: Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            background: None,
            outline: None,
            shadow: None,
            ..TextStyle::default()
        },
    };
    let edit = batch(
        "op_nan",
        &project,
        vec![EditOperation::ReplaceSource {
            clip_id: ItemId::new("itm_caption").unwrap(),
            source: Box::new(source),
            source_mapping: None,
        }],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "TEXT_STYLE");

    let mut overflow = project;
    overflow.project.revision = u64::MAX;
    let edit = batch(
        "op_overflow",
        &overflow,
        vec![EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_caption").unwrap(),
            enabled: false,
        }],
    );
    assert_rejected(apply_edit_batch(&overflow, &edit), "INVALID_EDIT_BATCH");

    let unsafe_time: RationalTime = serde_json::from_str(&format!(
        r#"{{"value":{},"timescale":600}}"#,
        MAX_SAFE_INTEGER + 1
    ))
    .unwrap();
    let edit = batch(
        "op_unsafe_time",
        &sample_project(),
        vec![EditOperation::MoveClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
            record_start: unsafe_time,
        }],
    );
    assert_rejected(
        apply_edit_batch(&sample_project(), &edit),
        "INVALID_EDIT_BATCH",
    );
    assert_eq!(time(0).value, 0);
}
