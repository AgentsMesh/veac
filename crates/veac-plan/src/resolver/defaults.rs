use veac_ir::{
    Anchor, Animatable, AudioProperties, BlendMode, Compositing, Length, LengthUnit, PitchPolicy,
    Placement, Point, Transform2D, Vec2, VisualProperties,
};

use crate::{EffectiveAudioProperties, EffectiveVisualProperties};

pub(super) fn visual(authored: Option<&VisualProperties>) -> EffectiveVisualProperties {
    let value = authored.cloned().unwrap_or_else(default_visual);
    EffectiveVisualProperties {
        placement: value.placement,
        frame: value.frame,
        transform: value.transform,
        opacity: value.opacity,
        compositing: value.compositing,
        masks: value.masks,
        track_matte: None,
        card: value.card,
        color_pipeline: None,
    }
}

pub(super) fn audio(authored: Option<&AudioProperties>) -> EffectiveAudioProperties {
    let value = authored.cloned().unwrap_or_else(default_audio);
    EffectiveAudioProperties {
        gain: value.gain,
        pan: value.pan,
        muted: value.muted,
        normalize: value.normalize,
        pitch_policy: value.pitch_policy,
        processors: value.processors,
        crossfade: value.crossfade,
        sidechain: None,
    }
}

fn default_visual() -> VisualProperties {
    let zero_offset = Point {
        x: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: 0.0,
            unit: LengthUnit::Pixels,
        },
    };
    VisualProperties {
        placement: Placement::Anchor {
            anchor: Anchor::Center,
            inset: Vec2 { x: 0.0, y: 0.0 },
        },
        frame: None,
        transform: Transform2D {
            position: Animatable::constant(zero_offset),
            scale: Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
            flip_horizontal: false,
            flip_vertical: false,
            rotation_degrees: Animatable::constant(0.0),
            anchor: Vec2 { x: 0.5, y: 0.5 },
            crop: None,
        },
        opacity: Animatable::constant(1.0),
        compositing: Compositing {
            z_index: 0,
            blend_mode: BlendMode::Normal,
        },
        masks: Vec::new(),
        card: None,
        color_pipeline: None,
    }
}

fn default_audio() -> AudioProperties {
    AudioProperties {
        gain: Animatable::constant(1.0),
        pan: Animatable::constant(0.0),
        muted: false,
        normalize: false,
        pitch_policy: PitchPolicy::Preserve,
        processors: Vec::new(),
        crossfade: None,
    }
}
