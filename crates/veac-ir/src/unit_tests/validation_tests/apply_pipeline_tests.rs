use super::*;
use crate::test_support::range;

use super::apply_target_tests::base_apply;

#[test]
fn apply_and_stage_ids_are_globally_unique() {
    let mut project = sample_project();
    let target = ApplyTarget::Layer {
        track_id: TrackId::new("trk_video").unwrap(),
    };
    let first = base_apply("apl_duplicate", "aps_duplicate", target.clone());
    let mut second = base_apply("apl_duplicate", "aps_duplicate", target);
    second.id = ApplyId::new("apl_second").unwrap();
    project.project.sequences[0].applies = vec![first, second];
    let codes = validation_codes(&project);
    assert_code(&codes, "DUPLICATE_APPLY_STAGE_ID");

    project.project.sequences[0].applies[1].id = ApplyId::new("apl_duplicate").unwrap();
    assert_code(&validation_codes(&project), "DUPLICATE_APPLY_ID");
}

#[test]
fn stages_are_nonempty_and_use_apply_local_ranges() {
    let mut project = sample_project();
    let mut apply = base_apply(
        "apl_pipeline",
        "aps_pipeline",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    apply.stages[0].active_range = Some(range(500, 200));
    project.project.sequences[0].applies.push(apply);
    assert_code(&validation_codes(&project), "APPLY_STAGE_RANGE");

    project.project.sequences[0].applies[0].stages.clear();
    assert_code(&validation_codes(&project), "APPLY_STAGE_COUNT");
}

#[test]
fn nested_effect_range_and_invalid_mix_are_rejected() {
    let mut project = sample_project();
    let mut apply = base_apply(
        "apl_effect",
        "aps_effect",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    let mut effect = project.project.sequences[0].tracks[0].clips[0].effects[0].clone();
    effect.id = EffectId::new("fx_apply").unwrap();
    effect.enable_range = Some(range(0, 300));
    apply.stages[0].operation = ApplyOperation::Effect { effect };
    apply.mix.opacity = Animatable::constant(2.0);
    project.project.sequences[0].applies.push(apply);
    let codes = validation_codes(&project);
    assert_code(&codes, "APPLY_EFFECT_RANGE");
    assert_code(&codes, "ANIMATION_VALUE");
}
