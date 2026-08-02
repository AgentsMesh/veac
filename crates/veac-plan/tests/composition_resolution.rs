mod support;

use support::*;
use veac_plan::{canonical::*, resolve, ResolvedApplyOperation, ResolvedApplyTarget};

#[test]
fn apply_targets_materialize_exact_nodes_and_absolute_stage_ranges() {
    let mut project = project();
    let sequence = &mut project.project.sequences[0];
    sequence.tracks[0].clips[0].visual = Some(visual_properties());
    sequence.tracks[0].clips[0].record_range.duration = time(300);
    let mut later = sequence.tracks[0].clips[0].clone();
    later.id = ItemId::new("itm_later").unwrap();
    later.record_range.start = time(300);
    later.record_range.duration = time(300);
    sequence.tracks[0].clips.push(later);

    let mut upper = track(
        "trk_upper",
        TrackKind::Visual,
        10,
        vec![generated_clip(
            "itm_upper",
            Generator::Solid {
                color: Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
            },
            0,
        )],
    );
    upper.clips[0].record_range.duration = time(600);
    upper.clips[0].visual = Some(visual_properties());
    sequence.tracks.push(upper);

    sequence.applies = vec![
        apply(
            "apl_item",
            ApplyTarget::ItemSet {
                item_ids: vec![ItemId::new("itm_video").unwrap()],
            },
        ),
        apply(
            "apl_layer",
            ApplyTarget::Layer {
                track_id: TrackId::new("trk_video").unwrap(),
            },
        ),
        apply(
            "apl_band",
            ApplyTarget::CompositeBand {
                from_track_id: TrackId::new("trk_video").unwrap(),
                through_track_id: TrackId::new("trk_upper").unwrap(),
            },
        ),
    ];

    let plan = resolve(&project, None).unwrap().remove(0);
    let resolved = plan.sequences.last().unwrap();
    assert_eq!(resolved.applies.len(), 3);
    assert_eq!(resolved.applies[0].source_order, 0);
    match &resolved.applies[0].target {
        ResolvedApplyTarget::ItemSet { items } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].item_id.as_str(), "itm_video");
        }
        other => panic!("unexpected target: {other:?}"),
    }
    match &resolved.applies[1].target {
        ResolvedApplyTarget::Layer { item_ids, .. } => {
            assert_eq!(item_ids.len(), 2);
            assert!(item_ids.iter().any(|id| id.as_str() == "itm_later"));
        }
        other => panic!("unexpected target: {other:?}"),
    }
    match &resolved.applies[2].target {
        ResolvedApplyTarget::CompositeBand { track_ids, .. } => {
            assert_eq!(track_ids.len(), 2);
            assert_eq!(track_ids[0].as_str(), "trk_video");
            assert_eq!(track_ids[1].as_str(), "trk_upper");
        }
        other => panic!("unexpected target: {other:?}"),
    }
    let stage = &resolved.applies[0].stages[0];
    assert_eq!(stage.active_range.start, time(90));
    assert!(matches!(
        stage.operation,
        ResolvedApplyOperation::Effect { .. }
    ));
}

#[test]
fn disabled_applies_and_stages_leave_only_executable_ordered_work() {
    let mut project = project();
    let sequence = &mut project.project.sequences[0];
    let mut disabled = apply(
        "apl_disabled",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    disabled.enabled = false;
    let mut empty = apply(
        "apl_empty",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    empty.stages[0].enabled = false;
    let mut partial = apply(
        "apl_partial",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    let mut skipped = partial.stages[0].clone();
    skipped.id = ApplyStageId::new("aps_partial_skipped").unwrap();
    skipped.enabled = false;
    let ApplyOperation::Effect { effect } = &mut skipped.operation else {
        unreachable!();
    };
    effect.id = EffectId::new("fx_partial_skipped").unwrap();
    partial.stages.insert(0, skipped);
    sequence.applies = vec![disabled, empty, partial];

    let plan = resolve(&project, None).unwrap().remove(0);
    let applies = &plan.sequences[0].applies;
    assert_eq!(applies.len(), 1);
    assert_eq!(applies[0].id.as_str(), "apl_partial");
    assert_eq!(applies[0].source_order, 2);
    assert_eq!(applies[0].stages.len(), 1);
    assert_eq!(applies[0].stages[0].id.as_str(), "aps_partial");
}

#[test]
fn apply_matte_is_projected_to_its_apply_consumer() {
    let mut project = project();
    let mut matte = generated_clip("itm_apply_matte", Generator::Transparent, 0);
    matte.record_range.duration = time(600);
    matte.visual = Some(visual_properties());
    let sequence = &mut project.project.sequences[0];
    sequence
        .tracks
        .push(track("trk_apply_matte", TrackKind::Visual, 20, vec![matte]));
    sequence.applies.push(apply(
        "apl_matted",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    ));
    add_apply_matte(
        &mut project,
        "rel_apply_matte",
        "itm_apply_matte",
        "apl_matted",
        matte_parameters(TrackMatteMode::Luma, true),
    );

    let plan = resolve(&project, None).unwrap().remove(0);
    let matte = plan.sequences[0].applies[0].matte.as_ref().unwrap();
    assert_eq!(matte.relation_id.as_str(), "rel_apply_matte");
    assert_eq!(matte.source_clip_id.as_str(), "itm_apply_matte");
    assert_eq!(matte.mode, TrackMatteMode::Luma);
    assert!(matte.invert);
}
