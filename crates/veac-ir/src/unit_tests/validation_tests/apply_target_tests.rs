use super::*;
use crate::test_support::range;

#[test]
fn layer_band_and_item_set_targets_are_closed_and_local() {
    let mut valid = sample_project();
    valid.project.sequences[0].applies.push(base_apply(
        "apl_layer",
        "aps_layer",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    ));
    assert!(validate(&valid).is_ok());

    let mut missing = valid.clone();
    missing.project.sequences[0].applies[0].target = ApplyTarget::Layer {
        track_id: TrackId::new("trk_absent").unwrap(),
    };
    assert_code(&validation_codes(&missing), "APPLY_TRACK_NOT_FOUND");

    let mut audio = crate::test_support::linked_project();
    audio.project.sequences[0].applies.push(base_apply(
        "apl_audio",
        "aps_audio",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_audio").unwrap(),
        },
    ));
    assert_code(&validation_codes(&audio), "APPLY_TARGET_TYPE");
}

#[test]
fn item_set_must_be_nonempty_sorted_unique_and_overlapping() {
    let mut project = sample_project();
    project.project.sequences[0].applies.push(base_apply(
        "apl_items",
        "aps_items",
        ApplyTarget::ItemSet {
            item_ids: vec![
                ItemId::new("itm_video").unwrap(),
                ItemId::new("itm_video").unwrap(),
            ],
        },
    ));
    assert_code(&validation_codes(&project), "APPLY_ITEM_SET_ORDER");

    project.project.sequences[0].applies[0].target = ApplyTarget::ItemSet { item_ids: vec![] };
    assert_code(&validation_codes(&project), "APPLY_ITEM_SET_EMPTY");

    project.project.sequences[0].applies[0].target = ApplyTarget::ItemSet {
        item_ids: vec![ItemId::new("itm_absent").unwrap()],
    };
    assert_code(&validation_codes(&project), "APPLY_ITEM_NOT_FOUND");
}

#[test]
fn ranges_and_band_order_fail_closed() {
    let mut project = sample_project();
    let mut apply = base_apply(
        "apl_band",
        "aps_band",
        ApplyTarget::CompositeBand {
            from_track_id: TrackId::new("trk_captions").unwrap(),
            through_track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    apply.record_range = range(500, 200);
    project.project.sequences[0].applies.push(apply);
    let codes = validation_codes(&project);
    assert_code(&codes, "APPLY_BAND_ORDER");
    assert_code(&codes, "APPLY_RANGE");
}

pub(super) fn base_apply(id: &str, stage: &str, target: ApplyTarget) -> Apply {
    Apply {
        id: ApplyId::new(id).unwrap(),
        enabled: true,
        record_range: range(0, 600),
        target,
        stages: vec![ApplyStage {
            id: ApplyStageId::new(stage).unwrap(),
            enabled: true,
            active_range: None,
            operation: ApplyOperation::Color {
                pipeline: empty_pipeline(),
            },
        }],
        mix: ApplyMix::default(),
    }
}

pub(super) fn empty_pipeline() -> ColorPipeline {
    let space = ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    };
    ColorPipeline {
        input: space,
        working: space,
        output: space,
        stages: vec![],
    }
}
