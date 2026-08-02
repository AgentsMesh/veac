use crate::test_support::sample_project;

use super::*;

#[test]
fn typed_curve_targets_report_each_missing_component() {
    let project = sample_project();
    let mut visual = project.project.sequences[0].tracks[0].clips[0].clone();
    let mut caption = project.project.sequences[0].tracks[1].clips[0].clone();
    let mut no_visual = visual.clone();
    no_visual.visual = None;
    assert!(number(&mut no_visual, &NumberCurveTarget::VisualOpacity).is_err());
    assert!(point(&mut no_visual, PointCurveTarget::VisualPosition).is_err());
    assert!(vec2(&mut no_visual, &Vec2CurveTarget::VisualScale).is_err());

    let mut no_audio = caption.clone();
    no_audio.audio = None;
    assert!(number(&mut no_audio, &NumberCurveTarget::AudioGain).is_err());
    assert!(number(&mut visual, &NumberCurveTarget::TextReveal).is_err());
    assert!(number(&mut caption, &NumberCurveTarget::TextOpacity).is_err());
    assert!(number(
        &mut visual,
        &NumberCurveTarget::MaskFeatherPixels { mask_index: 99 }
    )
    .is_err());
}

#[test]
fn effect_curve_target_rejects_missing_and_nonnumeric_parameters() {
    let project = sample_project();
    let mut clip = project.project.sequences[0].tracks[0].clips[0].clone();
    let effect_id = clip.effects[0].id.clone();
    assert!(number(
        &mut clip,
        &NumberCurveTarget::EffectParameter {
            effect_id: EffectId::new("fx_absent").unwrap(),
            name: "brightness".to_owned(),
        },
    )
    .is_err());
    assert!(number(
        &mut clip,
        &NumberCurveTarget::EffectParameter {
            effect_id: effect_id.clone(),
            name: "absent".to_owned(),
        },
    )
    .is_err());
    clip.effects[0]
        .parameters
        .insert("flag".to_owned(), ParameterValue::Boolean { value: true });
    assert!(number(
        &mut clip,
        &NumberCurveTarget::EffectParameter {
            effect_id,
            name: "flag".to_owned(),
        },
    )
    .is_err());
}
