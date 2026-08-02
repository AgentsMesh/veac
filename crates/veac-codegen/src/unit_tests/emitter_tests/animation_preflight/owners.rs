mod support;

use veac_plan::canonical::*;
use veac_plan::ResolvedRenderPlan;

#[derive(Debug, Clone, Copy)]
pub(super) enum Target {
    VisualPosition,
    VisualScale,
    VisualRotation,
    VisualOpacity,
    AudioGain,
    AudioPan,
    MaskPosition,
    MaskScale,
    MaskRotation,
    MaskFeather,
    MaskExpansion,
    EffectNumber,
    TextReveal,
    TextOpacity,
    TextPosition,
    TextScale,
    TextRotation,
}

impl Target {
    pub(super) const ALL: [Self; 17] = [
        Self::VisualPosition,
        Self::VisualScale,
        Self::VisualRotation,
        Self::VisualOpacity,
        Self::AudioGain,
        Self::AudioPan,
        Self::MaskPosition,
        Self::MaskScale,
        Self::MaskRotation,
        Self::MaskFeather,
        Self::MaskExpansion,
        Self::EffectNumber,
        Self::TextReveal,
        Self::TextOpacity,
        Self::TextPosition,
        Self::TextScale,
        Self::TextRotation,
    ];

    fn text(self) -> bool {
        matches!(
            self,
            Self::TextReveal
                | Self::TextOpacity
                | Self::TextPosition
                | Self::TextScale
                | Self::TextRotation
        )
    }
}

pub(super) fn malformed(target: Target) -> ResolvedRenderPlan {
    malformed_with(target, false)
}

pub(super) fn malformed_value(target: Target) -> ResolvedRenderPlan {
    malformed_with(target, true)
}

fn malformed_with(target: Target, bad_value: bool) -> ResolvedRenderPlan {
    let mut plan = support::plan(target.text());
    if target.text() {
        malformed_text(&mut plan, target, bad_value);
        return plan;
    }
    let clip = &mut plan.sequences[0].tracks[0].clips[0];
    match target {
        Target::VisualPosition => {
            clip.visual.as_mut().unwrap().transform.position = invalid(point(f64::NAN), bad_value)
        }
        Target::VisualScale => {
            clip.visual.as_mut().unwrap().transform.scale =
                invalid(Vec2 { x: 0.0, y: 1.0 }, bad_value)
        }
        Target::VisualRotation => {
            clip.visual.as_mut().unwrap().transform.rotation_degrees = invalid(f64::NAN, bad_value)
        }
        Target::VisualOpacity => clip.visual.as_mut().unwrap().opacity = invalid(2.0, bad_value),
        Target::AudioGain | Target::AudioPan => {
            let audio = clip.audio.get_or_insert_with(support::audio);
            match target {
                Target::AudioGain => audio.gain = invalid(-1.0, bad_value),
                Target::AudioPan => audio.pan = invalid(2.0, bad_value),
                _ => unreachable!(),
            }
        }
        Target::MaskPosition
        | Target::MaskScale
        | Target::MaskRotation
        | Target::MaskFeather
        | Target::MaskExpansion => {
            clip.visual.as_mut().unwrap().masks = vec![support::mask()];
            let mask = &mut clip.visual.as_mut().unwrap().masks[0];
            match target {
                Target::MaskPosition => mask.position = invalid(Vec2 { x: 2.0, y: 0.5 }, bad_value),
                Target::MaskScale => mask.scale = invalid(Vec2 { x: 0.0, y: 1.0 }, bad_value),
                Target::MaskRotation => mask.rotation_degrees = invalid(f64::NAN, bad_value),
                Target::MaskFeather => mask.feather_pixels = invalid(-1.0, bad_value),
                Target::MaskExpansion => mask.expansion_pixels = invalid(f64::NAN, bad_value),
                _ => unreachable!(),
            }
        }
        Target::EffectNumber => clip.effects.push(support::effect(
            clip.record_range.duration,
            invalid(f64::NAN, bad_value),
        )),
        _ => unreachable!(),
    }
    plan
}

pub(super) fn invalid_stagger() -> ResolvedRenderPlan {
    let mut plan = support::plan(true);
    let animation = support::text_animation(&mut plan);
    animation.stagger = RationalTime {
        value: 0,
        timescale: 0,
    };
    plan
}

fn malformed_text(plan: &mut ResolvedRenderPlan, target: Target, bad_value: bool) {
    let animation = support::text_animation(plan);
    match target {
        Target::TextReveal => animation.reveal = invalid(2.0, bad_value),
        Target::TextOpacity => animation.opacity = invalid(-1.0, bad_value),
        Target::TextPosition => {
            animation.transform.position_offset = invalid(point(f64::NAN), bad_value)
        }
        Target::TextScale => {
            animation.transform.scale = invalid(Vec2 { x: 0.0, y: 1.0 }, bad_value)
        }
        Target::TextRotation => animation.transform.rotation_degrees = invalid(f64::NAN, bad_value),
        _ => unreachable!(),
    }
}

fn invalid<T>(bad_value: T, value: bool) -> Animatable<T> {
    if value {
        Animatable::constant(bad_value)
    } else {
        Animatable::Keyframes { keyframes: vec![] }
    }
}

fn point(value: f64) -> Point {
    Point {
        x: Length {
            value,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    }
}
