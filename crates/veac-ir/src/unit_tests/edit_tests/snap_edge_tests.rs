use std::collections::BTreeMap;

use super::*;

#[test]
fn snap_rejects_each_timebase_mismatch_and_an_unsafe_grid_ceil() {
    let project = sample_project();
    let base = SnapRequest {
        sequence_id: SequenceId::new("seq_main").unwrap(),
        time: time(10),
        tolerance: time(20),
        include_clip_edges: false,
        include_frame_grid: true,
        excluded_items: vec![],
    };
    for request in [
        SnapRequest {
            time: RationalTime::new(10, 1_000).unwrap(),
            ..base.clone()
        },
        SnapRequest {
            tolerance: RationalTime::new(20, 1_000).unwrap(),
            ..base.clone()
        },
    ] {
        let error = snap_candidates(&project, &request).unwrap_err();
        assert_eq!(error.code, "INVALID_SNAP_REQUEST");
        assert!(error.message.contains("project timebase"));
    }

    let request = SnapRequest {
        time: RationalTime::new(MAX_SAFE_INTEGER as i64, 600).unwrap(),
        ..base
    };
    let error = snap_candidates(&project, &request).unwrap_err();
    assert_eq!(error.code, "INVALID_SNAP_REQUEST");
    assert!(error.message.contains("frame grid time is unsafe"));
}

#[test]
fn snap_rejects_a_frame_step_outside_the_render_budget() {
    let project = huge_frame_step_project();
    let request = SnapRequest {
        sequence_id: SequenceId::new("seq_huge_grid").unwrap(),
        time: RationalTime::zero(u32::MAX).unwrap(),
        tolerance: RationalTime::zero(u32::MAX).unwrap(),
        include_clip_edges: false,
        include_frame_grid: true,
        excluded_items: vec![],
    };
    let error = snap_candidates(&project, &request).unwrap_err();
    assert_eq!(error.code, "SEQUENCE_SETTINGS");
    assert!(error.message.contains("render budget"));
}

fn huge_frame_step_project() -> ProjectEnvelope {
    let sequence_id = SequenceId::new("seq_huge_grid").unwrap();
    ProjectEnvelope::new(Project {
        id: ProjectId::new("prj_huge_grid").unwrap(),
        revision: 0,
        timebase: u32::MAX,
        entry_sequence_id: sequence_id.clone(),
        render_configs: vec![RenderConfig {
            id: RenderConfigId::new("out_huge_grid").unwrap(),
            sequence_id: sequence_id.clone(),
            raster: Some(RasterSettings {
                width: 16,
                height: 16,
                frame_rate: Rational::new(1, 1).unwrap(),
                captions: CaptionOutput::Discard,
            }),
            deliverables: vec![Deliverable {
                id: DeliverableId::new("dlv_huge_grid").unwrap(),
                target: DeliverableTarget::File {
                    name: "huge.mp4".to_owned(),
                },
                kind: DeliverableKind::Video(VideoDeliverable::default()),
            }],
        }],
        materials: vec![],
        multicam_groups: vec![],
        annotations: vec![],
        relations: vec![],
        sequences: vec![Sequence {
            id: sequence_id,
            name: "Huge grid".to_owned(),
            settings: SequenceSettings {
                width: 16,
                height: 16,
                frame_rate: Rational::new(1, u32::MAX).unwrap(),
                sample_rate: 48_000,
            },
            tracks: vec![],
            applies: vec![],
            metadata: BTreeMap::new(),
        }],
        applied_operations: vec![],
        metadata: BTreeMap::new(),
    })
}
