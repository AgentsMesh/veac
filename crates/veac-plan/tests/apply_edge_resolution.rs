mod support;

use support::*;
use veac_plan::{canonical::*, resolve, ResolvedApplyOperation, ResolvedApplyTarget};

fn rec709() -> ColorSpace {
    ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    }
}

#[test]
fn empty_color_pipeline_remains_an_explicit_apply_stage() {
    let mut project = project();
    let sequence = &mut project.project.sequences[0];
    let mut value = apply(
        "apl_empty_color",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    value.stages[0].operation = ApplyOperation::Color {
        pipeline: ColorPipeline {
            input: rec709(),
            working: rec709(),
            output: rec709(),
            stages: Vec::new(),
        },
    };
    sequence.applies.push(value);

    let plan = resolve(&project, None).unwrap().remove(0);
    let operation = &plan.sequences.last().unwrap().applies[0].stages[0].operation;
    let ResolvedApplyOperation::Color { pipeline } = operation else {
        panic!("expected color operation")
    };
    assert!(pipeline.stages.is_empty());
}

#[test]
fn item_set_apply_keeps_targets_across_tracks() {
    let mut project = project();
    let sequence = &mut project.project.sequences[0];
    sequence.tracks.push(track(
        "trk_visual",
        TrackKind::Visual,
        10,
        vec![media_clip("itm_visual", "med_video", 0)],
    ));
    let item_ids = vec![
        sequence.tracks[0].clips[0].id.clone(),
        sequence.tracks[1].clips[0].id.clone(),
    ];
    sequence.applies.push(apply(
        "apl_items",
        ApplyTarget::ItemSet {
            item_ids: item_ids.clone(),
        },
    ));

    let plan = resolve(&project, None).unwrap().remove(0);
    let target = &plan.sequences.last().unwrap().applies[0].target;
    let ResolvedApplyTarget::ItemSet { items } = target else {
        panic!("expected item-set target")
    };
    let resolved: Vec<_> = items.iter().map(|item| item.item_id.clone()).collect();
    assert_eq!(resolved, item_ids);
}

#[test]
fn layer_apply_keeps_disjoint_ranges_and_ignores_disabled_effects() {
    let mut project = project();
    let sequence = &mut project.project.sequences[0];
    sequence.tracks[0].placement_mode = PlacementMode::Free;
    sequence.tracks[0]
        .clips
        .push(media_clip("itm_visual_second", "med_video", 1_200));

    let mut value = apply(
        "apl_disjoint",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    value.record_range = range(0, 1_800);
    value.stages[0].active_range = None;
    let mut disabled = value.stages[0].clone();
    disabled.id = ApplyStageId::new("aps_disjoint_disabled").unwrap();
    let ApplyOperation::Effect { effect } = &mut disabled.operation else {
        unreachable!()
    };
    effect.id = EffectId::new("fx_disjoint_disabled").unwrap();
    effect.enabled = false;
    value.stages.push(disabled);
    sequence.applies.push(value);

    let plan = resolve(&project, None).unwrap().remove(0);
    let resolved = &plan.sequences.last().unwrap().applies[0];
    let ResolvedApplyTarget::Layer { active_ranges, .. } = &resolved.target else {
        panic!("expected layer target")
    };
    assert_eq!(active_ranges.len(), 2);
    assert_eq!(resolved.stages.len(), 1);
}
