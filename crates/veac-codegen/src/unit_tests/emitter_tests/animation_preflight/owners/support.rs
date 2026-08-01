use std::collections::BTreeMap;

use veac_plan::canonical::*;
use veac_plan::{EffectiveAudioProperties, ResolvedClipSource, ResolvedEffect, ResolvedRenderPlan};

use super::super::super::support::{fixture, resolved, text_fixture, time};

pub(super) fn plan(text: bool) -> ResolvedRenderPlan {
    if text {
        resolved(&text_fixture(false))
    } else {
        resolved(&fixture())
    }
}

pub(super) fn text_animation(plan: &mut ResolvedRenderPlan) -> &mut TextAnimation {
    let content = plan
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .flat_map(|track| &mut track.clips)
        .find_map(|clip| match &mut clip.source {
            ResolvedClipSource::Text { content } => Some(content),
            _ => None,
        })
        .unwrap();
    content
        .styled_mut()
        .unwrap()
        .animation
        .get_or_insert_with(|| TextAnimation {
            granularity: TextGranularity::Whole,
            transform: TextUnitTransform::default(),
            reveal: Animatable::constant(1.0),
            highlight: None,
            opacity: Animatable::constant(1.0),
            stagger: time(0),
        })
}

pub(super) fn audio() -> EffectiveAudioProperties {
    EffectiveAudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: vec![],
        crossfade: None,
        sidechain: None,
    }
}

pub(super) fn mask() -> Mask {
    Mask {
        shape: MaskShape::Circle,
        position: Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
        scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
        rotation_degrees: Animatable::constant(0.0),
        feather_pixels: Animatable::constant(0.0),
        expansion_pixels: Animatable::constant(0.0),
        invert: false,
    }
}

pub(super) fn effect(duration: RationalTime, value: Animatable<f64>) -> ResolvedEffect {
    ResolvedEffect {
        id: EffectId::new("fx_curve_preflight").unwrap(),
        effect_type: "video.color_adjust".to_owned(),
        active_range: TimeRange {
            start: time(0),
            duration,
        },
        parameters: BTreeMap::from([(
            "brightness".to_owned(),
            ParameterValue::NumberCurve { value },
        )]),
    }
}
