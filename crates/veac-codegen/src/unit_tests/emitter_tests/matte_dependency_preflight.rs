use std::panic::{catch_unwind, AssertUnwindSafe};

use veac_plan::canonical::*;
use veac_plan::{ResolvedApplyItem, ResolvedApplyTarget, ResolvedClipSource, ResolvedMatte};

use super::composition_advanced::advanced_plan;
use super::support::{bindings, emit_video_command};

#[test]
fn mixed_clip_and_item_apply_cycle_fails_before_emission_recurses() {
    let mut plan = advanced_plan();
    let target = item(&plan, 1, 0);
    let producer = item(&plan, 0, 0);
    set_item_apply(&mut plan, vec![target], producer);

    let result = catch_unwind(AssertUnwindSafe(|| command(&plan)));
    let error = result.expect("preflight must not recurse").unwrap_err();
    assert_code(&error, "PLAN_MATTE_CYCLE");
}

#[test]
fn one_item_may_depend_on_two_matte_sources_without_a_false_cycle() {
    let mut plan = advanced_plan();
    let extra = add_source(&mut plan, "trk_branch_source", "itm_branch_source", 6);
    let target = item(&plan, 0, 0);
    set_item_apply(&mut plan, vec![target], extra);
    command(&plan).unwrap();
}

#[test]
fn dormant_item_set_member_does_not_execute_apply_or_matte() {
    let mut with_dormant = advanced_plan();
    add_source(&mut with_dormant, "trk_dormant", "itm_z_dormant", 6);
    let dormant = &mut with_dormant.sequences[0].tracks.last_mut().unwrap().clips[0];
    dormant.record_range = range(480, 120);
    dormant.source = ResolvedClipSource::Generated {
        generator: Generator::Transparent,
    };
    dormant.source_mapping = None;
    let apply = &mut with_dormant.sequences[0].applies[0];
    apply.record_range = range(120, 480);
    apply.stages[0].active_range = range(120, 360);
    let live = item(&with_dormant, 0, 0);
    let dormant_item = ResolvedApplyItem {
        track_id: TrackId::new("trk_dormant").unwrap(),
        item_id: ItemId::new("itm_z_dormant").unwrap(),
        active_range: range(480, 120),
    };
    let producer = item(&with_dormant, 1, 0);
    set_item_apply(
        &mut with_dormant,
        vec![live.clone(), dormant_item],
        producer,
    );
    let mut live_only = with_dormant.clone();
    let producer = item(&live_only, 1, 0);
    set_item_apply(&mut live_only, vec![live], producer);
    assert_eq!(
        command(&with_dormant).unwrap(),
        command(&live_only).unwrap()
    );
}

#[test]
fn matte_depth_counts_edges_with_an_exact_64_65_boundary() {
    command(&depth_plan(64)).expect("64 matte edges are executable");

    let result = catch_unwind(AssertUnwindSafe(|| command(&depth_plan(65))));
    let error = result.expect("preflight must not recurse").unwrap_err();
    assert_code(&error, "PLAN_MATTE_DEPTH_EXCEEDED");
}

fn depth_plan(edges: usize) -> veac_plan::ResolvedRenderPlan {
    let mut plan = advanced_plan();
    plan.sequences[0].applies.clear();
    let track = &mut plan.sequences[0].tracks[1];
    for index in 1..edges {
        let mut clip = track.clips[0].clone();
        clip.id = ItemId::new(format!("itm_matte_depth_{index:03}")).unwrap();
        clip.source_order = index as u32;
        clip.visual.as_mut().unwrap().track_matte = None;
        track.clips.push(clip);
    }
    for index in 0..track.clips.len().saturating_sub(1) {
        let source_clip_id = track.clips[index + 1].id.clone();
        track.clips[index].visual.as_mut().unwrap().track_matte = Some(matte(
            &format!("rel_matte_depth_{index:03}"),
            source_clip_id,
        ));
    }
    plan
}

fn add_source(
    plan: &mut veac_plan::ResolvedRenderPlan,
    track_id: &str,
    item_id: &str,
    order: i32,
) -> ResolvedApplyItem {
    let mut track = plan.sequences[0].tracks[1].clone();
    track.id = TrackId::new(track_id).unwrap();
    track.order = order;
    track.source_order = plan.sequences[0].tracks.len() as u32;
    track.clips[0].id = ItemId::new(item_id).unwrap();
    track.clips[0].source_order = 0;
    track.clips[0].visual.as_mut().unwrap().track_matte = None;
    let active_range = track.clips[0].record_range;
    let value = ResolvedApplyItem {
        track_id: track.id.clone(),
        item_id: track.clips[0].id.clone(),
        active_range,
    };
    plan.sequences[0].tracks.push(track);
    value
}

fn set_item_apply(
    plan: &mut veac_plan::ResolvedRenderPlan,
    mut items: Vec<ResolvedApplyItem>,
    source: ResolvedApplyItem,
) {
    items.sort_by(|left, right| left.item_id.cmp(&right.item_id));
    let apply = &mut plan.sequences[0].applies[0];
    apply.target = ResolvedApplyTarget::ItemSet { items };
    apply.matte = Some(matte("rel_item_apply_matte", source.item_id));
}

fn item(plan: &veac_plan::ResolvedRenderPlan, track: usize, clip: usize) -> ResolvedApplyItem {
    let track = &plan.sequences[0].tracks[track];
    let clip = &track.clips[clip];
    ResolvedApplyItem {
        track_id: track.id.clone(),
        item_id: clip.id.clone(),
        active_range: plan.sequences[0].applies[0].record_range,
    }
}

fn matte(id: &str, source_clip_id: ItemId) -> ResolvedMatte {
    ResolvedMatte {
        relation_id: RelationId::new(id).unwrap(),
        source_clip_id,
        mode: TrackMatteMode::Alpha,
        invert: false,
    }
}

fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(
        RationalTime::new(start, 600).unwrap(),
        RationalTime::new(duration, 600).unwrap(),
    )
    .unwrap()
}

fn command(
    plan: &veac_plan::ResolvedRenderPlan,
) -> Result<veac_codegen::emitter::BackendCommand, veac_codegen::emitter::CodegenErrors> {
    emit_video_command(plan, &bindings(plan))
}

fn assert_code(error: &veac_codegen::emitter::CodegenErrors, code: &str) {
    assert!(error.diagnostics().iter().any(|value| value.code == code));
}
