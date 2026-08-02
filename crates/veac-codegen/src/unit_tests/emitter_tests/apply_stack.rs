use veac_plan::canonical::{ApplyId, ApplyStageId, ItemId, TrackId};
use veac_plan::{ResolvedApply, ResolvedApplyItem, ResolvedApplyTarget};

use super::composition_advanced::{advanced_plan, blur_stage, default_mix};
use super::support::{bindings, emit_video_command};

#[test]
fn same_layer_applies_are_serial_and_reuse_the_checkpoint() {
    let mut plan = advanced_plan();
    let mut second = plan.sequences[0].applies[0].clone();
    second.id = ApplyId::new("apl_second").unwrap();
    second.source_order = 1;
    second.stages[0].id = ApplyStageId::new("aps_second").unwrap();
    plan.sequences[0].applies.push(second);

    let graph = graph(&plan);
    assert_eq!(
        graph.matches("blend=all_expr='if(gte(T").count(),
        2,
        "{graph}"
    );
    assert!(graph.contains("applyprocessedbandsplitv"), "{graph}");
    assert!(graph.matches("gblur@").count() >= 2, "{graph}");
}

#[test]
fn same_origin_layer_and_composite_band_reuse_processed_result() {
    let mut plan = advanced_plan();
    let sequence = &mut plan.sequences[0];
    let mut top = sequence.tracks[0].clone();
    top.id = TrackId::new("trk_apply_top").unwrap();
    top.order = 10;
    top.source_order = 2;
    top.clips[0].id = ItemId::new("itm_apply_top").unwrap();
    top.clips[0].effects.clear();
    top.clips[0].visual.as_mut().unwrap().track_matte = None;
    let top_id = top.id.clone();
    sequence.tracks.push(top);

    let range = sequence.applies[0].record_range;
    sequence.applies.push(ResolvedApply {
        id: ApplyId::new("apl_outer_band").unwrap(),
        source_order: 1,
        record_range: range,
        target: ResolvedApplyTarget::CompositeBand {
            from_track_id: sequence.tracks[0].id.clone(),
            through_track_id: top_id,
            track_ids: sequence
                .tracks
                .iter()
                .map(|track| track.id.clone())
                .collect(),
            active_ranges: vec![range],
        },
        stages: vec![blur_stage("aps_outer_band", range)],
        mix: default_mix(),
        matte: None,
    });

    let graph = graph(&plan);
    assert_eq!(
        graph.matches("blend=all_expr='if(gte(T").count(),
        2,
        "{graph}"
    );
    assert!(graph.contains("applyprocessedbandsplitv"), "{graph}");
    assert!(graph.contains("applyunderlaysplitv"), "{graph}");
}

#[test]
fn nested_bands_checkpoint_and_propagate_each_inner_apply() {
    let mut plan = advanced_plan();
    let sequence = &mut plan.sequences[0];
    let mut middle = sequence.tracks[0].clone();
    middle.id = TrackId::new("trk_nested_middle").unwrap();
    middle.order = 10;
    middle.source_order = 2;
    middle.clips[0].id = ItemId::new("itm_nested_middle").unwrap();
    middle.clips[0].effects.clear();
    middle.clips[0].visual.as_mut().unwrap().track_matte = None;
    let middle_id = middle.id.clone();
    let middle_item = middle.clips[0].id.clone();
    let mut cap = middle.clone();
    cap.id = TrackId::new("trk_nested_cap").unwrap();
    cap.order = 11;
    cap.source_order = 3;
    cap.clips[0].id = ItemId::new("itm_nested_cap").unwrap();
    let cap_id = cap.id.clone();
    sequence.tracks.extend([middle, cap]);

    let range = sequence.applies[0].record_range;
    sequence.applies[0].target = ResolvedApplyTarget::Layer {
        track_id: middle_id.clone(),
        item_ids: vec![middle_item],
        active_ranges: vec![range],
    };
    let mut second = sequence.applies[0].clone();
    second.id = ApplyId::new("apl_nested_second").unwrap();
    second.source_order = 1;
    second.stages[0].id = ApplyStageId::new("aps_nested_second").unwrap();
    sequence.applies.push(second);
    sequence.applies.push(ResolvedApply {
        id: ApplyId::new("apl_nested_outer").unwrap(),
        source_order: 2,
        record_range: range,
        target: ResolvedApplyTarget::CompositeBand {
            from_track_id: sequence.tracks[0].id.clone(),
            through_track_id: cap_id,
            track_ids: sequence
                .tracks
                .iter()
                .map(|track| track.id.clone())
                .collect(),
            active_ranges: vec![range],
        },
        stages: vec![blur_stage("aps_nested_outer", range)],
        mix: default_mix(),
        matte: None,
    });

    let graph = graph(&plan);
    assert_eq!(graph.matches("gblur@").count(), 3, "{graph}");
    assert!(graph.contains("applynestedcheckpointv"), "{graph}");
    assert!(graph.contains("applynestedunderlaysplitv"), "{graph}");
}

#[test]
fn item_set_apply_resolves_the_selected_item_interval() {
    let mut plan = advanced_plan();
    let sequence = &mut plan.sequences[0];
    let range = sequence.applies[0].record_range;
    let track_id = sequence.tracks[0].id.clone();
    let item_id = sequence.tracks[0].clips[0].id.clone();
    sequence.applies[0].target = ResolvedApplyTarget::ItemSet {
        items: vec![ResolvedApplyItem {
            track_id,
            item_id,
            active_range: range,
        }],
    };

    let graph = graph(&plan);
    assert!(graph.contains("gblur@"), "{graph}");
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}
