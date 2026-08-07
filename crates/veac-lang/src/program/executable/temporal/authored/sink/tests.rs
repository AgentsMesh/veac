use super::*;
use crate::program::model::{TemporalApplyPath, TemporalItemPath};

fn item() -> TemporalItemPath {
    TemporalItemPath {
        project: "demo".into(),
        sequence: "main".into(),
        layer: "visual".into(),
        item: "clip".into(),
    }
}

fn apply() -> TemporalApplyPath {
    TemporalApplyPath {
        project: "demo".into(),
        sequence: "main".into(),
        apply: "grade".into(),
    }
}

#[test]
fn target_property_mismatches_and_apply_clip_clock_fail_closed() {
    for target in [
        Target::Clip(item()),
        Target::Text(item()),
        Target::ClipMask {
            clip: item(),
            mask_index: 0,
        },
        Target::Apply(apply()),
        Target::ApplyMask {
            apply: apply(),
            mask_index: 0,
        },
    ] {
        assert_eq!(lower(&target, Property::EffectParameter), Err(mismatch()));
    }
    assert_eq!(clip_sequence(&Target::Apply(apply())), Err(mismatch()));
}

#[test]
fn effect_targets_reject_parameters_outside_the_closed_catalog() {
    let clip = Target::ClipEffect {
        clip: item(),
        effect: "blur".into(),
        parameter: "unknown".into(),
    };
    assert_eq!(
        lower(&clip, Property::EffectParameter),
        Err(invalid_parameter())
    );
    let apply = Target::ApplyEffect {
        apply: apply(),
        stage: "soften".into(),
        effect: "blur".into(),
        parameter: "unknown".into(),
    };
    assert_eq!(
        lower(&apply, Property::EffectParameter),
        Err(invalid_parameter())
    );
}
