use std::panic::{catch_unwind, AssertUnwindSafe};

use veac_codegen::emitter::CodegenErrorKind;
use veac_plan::canonical::*;
use veac_plan::{
    ResolvedApply, ResolvedApplyOperation, ResolvedApplyStage, ResolvedApplyTarget, ResolvedMatte,
};

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn item_matte_and_apply_execute_in_render_order() {
    let mut plan = advanced_plan();
    let filter_graph = graph(&plan);
    for marker in [
        "alphaextract,format=gray16le",
        "mergeplanes=format=gbrap16le",
        "applycheckpointv",
        "applystagev",
        "gblur@",
        "sigma=2",
        "applyreplacev",
    ] {
        assert!(
            filter_graph.contains(marker),
            "missing {marker}: {filter_graph}"
        );
    }
    plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .track_matte
        .as_mut()
        .unwrap()
        .invert = true;
    assert!(graph(&plan).contains("matteinvertv"));
}

#[test]
fn untrusted_apply_target_and_matte_fail_preflight() {
    let mut item_matte = advanced_plan();
    item_matte.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .track_matte
        .as_mut()
        .unwrap()
        .source_clip_id = ItemId::new("itm_absent").unwrap();
    assert_invalid(&item_matte, "PLAN_MATTE_REFERENCE_INVALID");

    let mut apply_matte = advanced_plan();
    apply_matte.sequences[0].applies[0].matte = Some(ResolvedMatte {
        relation_id: RelationId::new("rel_apply_matte").unwrap(),
        source_clip_id: ItemId::new("itm_absent").unwrap(),
        mode: TrackMatteMode::Alpha,
        invert: false,
    });
    assert_invalid(&apply_matte, "PLAN_APPLY_MATTE_INVALID");

    let mut target = advanced_plan();
    let ResolvedApplyTarget::Layer { track_id, .. } = &mut target.sequences[0].applies[0].target
    else {
        unreachable!()
    };
    *track_id = TrackId::new("trk_absent").unwrap();
    assert_invalid(&target, "PLAN_APPLY_TARGET_INVALID");
}

#[test]
fn apply_masks_opacity_and_blend_are_executable() {
    let mut plan = advanced_plan();
    let mix = &mut plan.sequences[0].applies[0].mix;
    mix.opacity = Animatable::constant(0.5);
    mix.blend_mode = BlendMode::Multiply;
    mix.masks.push(Mask {
        shape: MaskShape::Circle,
        position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
        scale: Animatable::constant(Vec2 { x: 0.8, y: 0.8 }),
        rotation_degrees: Animatable::constant(0.0),
        invert: false,
        feather_pixels: Animatable::constant(0.0),
        expansion_pixels: Animatable::constant(0.0),
    });
    let graph = graph(&plan);
    for marker in [
        "applymixsplitv",
        "maskv",
        "opacityv",
        "blend=all_mode=multiply",
    ] {
        assert!(graph.contains(marker), "missing {marker}: {graph}");
    }
}

pub(crate) fn advanced_plan() -> veac_plan::ResolvedRenderPlan {
    let mut plan = resolved(&fixture());
    let target_track = plan.sequences[0].tracks[0].id.clone();
    let target_item = plan.sequences[0].tracks[0].clips[0].id.clone();
    let target_range = plan.sequences[0].tracks[0].clips[0].record_range;
    let mut source_track = plan.sequences[0].tracks[0].clone();
    source_track.id = TrackId::new("trk_matte_source").unwrap();
    source_track.order = 5;
    source_track.source_order = 1;
    source_track.clips[0].id = ItemId::new("itm_matte_source").unwrap();
    source_track.clips[0].effects.clear();
    source_track.clips[0].visual.as_mut().unwrap().track_matte = None;
    let target = &mut plan.sequences[0].tracks[0].clips[0];
    target.visual.as_mut().unwrap().track_matte = Some(ResolvedMatte {
        relation_id: RelationId::new("rel_advanced_matte").unwrap(),
        source_clip_id: source_track.clips[0].id.clone(),
        mode: TrackMatteMode::Alpha,
        invert: false,
    });
    let range = TimeRange::new(rt(120), rt(360)).unwrap();
    plan.sequences[0].tracks.push(source_track);
    plan.sequences[0].applies.push(ResolvedApply {
        id: ApplyId::new("apl_advanced").unwrap(),
        source_order: 0,
        record_range: range,
        target: ResolvedApplyTarget::Layer {
            track_id: target_track,
            item_ids: vec![target_item],
            active_ranges: vec![range],
        },
        stages: vec![blur_stage("aps_advanced", range)],
        mix: default_mix(),
        matte: None,
    });
    assert!(target_range.start <= range.start);
    plan
}

pub(super) fn blur_stage(id: &str, range: TimeRange) -> ResolvedApplyStage {
    ResolvedApplyStage {
        id: ApplyStageId::new(id).unwrap(),
        active_range: range,
        operation: ResolvedApplyOperation::Effect {
            effect: Effect::VideoBlur {
                radius: Animatable::constant(2.0),
            },
        },
    }
}

pub(super) fn default_mix() -> ApplyMix {
    ApplyMix {
        opacity: Animatable::constant(1.0),
        blend_mode: BlendMode::Normal,
        masks: Vec::new(),
    }
}

fn rt(value: i64) -> RationalTime {
    RationalTime::new(value, 600).unwrap()
}

fn graph(plan: &veac_plan::ResolvedRenderPlan) -> String {
    emit_video_command(plan, &bindings(plan))
        .unwrap()
        .filter_graph
        .unwrap()
}

fn assert_invalid(plan: &veac_plan::ResolvedRenderPlan, code: &str) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        emit_video_command(plan, &bindings(plan))
    }));
    let error = result.expect("preflight must not panic").unwrap_err();
    assert_eq!(error.diagnostics()[0].kind, CodegenErrorKind::InvalidPlan);
    assert!(error.diagnostics().iter().any(|value| value.code == code));
}
