use serde_json::json;
use veac_ir::{DeliverableId, RationalTime, SequenceId, TimeRange};

use crate::*;

#[test]
fn render_output_paths_are_bounded_and_revalidated() {
    let value = RenderOutputParameters::new(7, "frames/000007.png");
    assert_eq!(value.index, 7);
    value.validate().unwrap();

    for path in [String::new(), "bad\0path".to_owned()] {
        assert_eq!(
            RenderOutputParameters::new(0, path)
                .validate()
                .unwrap_err()
                .kind,
            ArtifactErrorKind::InvalidContract
        );
    }
    assert_eq!(
        RenderOutputParameters::new(0, "x".repeat(MAX_ARTIFACT_JSON_STRING_BYTES + 1))
            .validate()
            .unwrap_err()
            .kind,
        ArtifactErrorKind::ResourceLimit
    );
}

#[test]
fn render_segments_require_a_positive_half_open_range() {
    segment(0, 10).validate().unwrap();
    for value in [segment(-1, 10), segment(0, 0), segment(0, -1)] {
        assert_eq!(
            value.validate().unwrap_err().kind,
            ArtifactErrorKind::InvalidContract
        );
    }
}

#[test]
fn render_tasks_and_checkpoints_validate_both_scopes() {
    let task = task();
    task.validate().unwrap();
    RenderCheckpointParameters::Task(task.clone())
        .validate()
        .unwrap();
    RenderCheckpointParameters::Output(RenderOutputParameters::new(2, "master.mov"))
        .validate()
        .unwrap();

    let mut invalid_version = task.clone();
    invalid_version.contract_version = 0;
    assert_eq!(
        invalid_version.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
    let mut invalid_digest = task;
    invalid_digest.task_digest.value = "bad".to_owned();
    assert_eq!(
        invalid_digest.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

#[test]
fn render_parameter_json_is_closed_and_round_trips() {
    let values = [
        RenderCheckpointParameters::Task(task()),
        RenderCheckpointParameters::Output(RenderOutputParameters::new(3, "stem.wav")),
    ];
    for value in values {
        let encoded = serde_json::to_value(&value).unwrap();
        assert_eq!(
            serde_json::from_value::<RenderCheckpointParameters>(encoded).unwrap(),
            value
        );
    }

    assert!(serde_json::from_value::<RenderOutputParameters>(json!({
        "index": 0,
        "path": "master.mov",
        "extension": true
    }))
    .is_err());
    assert!(serde_json::from_value::<RenderCheckpointParameters>(json!({
        "scope": "output",
        "parameters": {"index": 0, "path": "master.mov"},
        "extension": true
    }))
    .is_err());
}

fn segment(start: i64, duration: i64) -> RenderSegmentParameters {
    RenderSegmentParameters {
        sequence_id: SequenceId::new("seq_main").unwrap(),
        range: TimeRange {
            start: RationalTime::new(start, 10).unwrap(),
            duration: RationalTime::new(duration, 10).unwrap(),
        },
        deliverable_id: DeliverableId::new("dlv_main").unwrap(),
        fidelity: RenderSegmentFidelity::ExactDeliveryMaster,
    }
}

pub(crate) fn valid_segment() -> RenderSegmentParameters {
    segment(0, 10)
}

fn task() -> RenderTaskParameters {
    RenderTaskParameters {
        contract_version: 1,
        deliverable_id: DeliverableId::new("dlv_main").unwrap(),
        phase: RenderTaskPhase::SecondPass,
        product: RenderProduct::VideoMaster,
        task_digest: ContentDigest::sha256(b"render task"),
    }
}
