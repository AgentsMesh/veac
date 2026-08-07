use super::{Effect, EffectDomain, EffectKind, EffectParameter, EffectParameterValue};
use crate::{Animatable, Color};

macro_rules! curve_ref {
    ($value:expr, $parameter:expr) => {
        match ($value, $parameter) {
            (Effect::VideoColorAdjust { brightness, .. }, EffectParameter::Brightness) => Some(brightness),
            (Effect::VideoColorAdjust { contrast, .. }, EffectParameter::Contrast) => Some(contrast),
            (Effect::VideoColorAdjust { saturation, .. }, EffectParameter::Saturation) => Some(saturation),
            (Effect::VideoBlur { radius }, EffectParameter::Radius) => Some(radius),
            (Effect::VideoSharpen { amount }, EffectParameter::Amount)
            | (Effect::VideoVignette { amount }, EffectParameter::Amount)
            | (Effect::VideoGrain { amount }, EffectParameter::Amount)
            | (Effect::VideoPluginReferenceMonochromeV1 { amount, .. }, EffectParameter::Amount) => Some(amount),
            (Effect::VideoChromaKey { similarity, .. }, EffectParameter::Similarity) => Some(similarity),
            (Effect::VideoChromaKey { blend, .. }, EffectParameter::Blend) => Some(blend),
            (Effect::VideoLumaKey { threshold, .. }, EffectParameter::Threshold) => Some(threshold),
            (Effect::VideoLumaKey { tolerance, .. }, EffectParameter::Tolerance) => Some(tolerance),
            (Effect::VideoLumaKey { softness, .. }, EffectParameter::Softness) => Some(softness),
            (Effect::VideoChromaSpill { amount, .. }, EffectParameter::Amount) => Some(amount),
            (Effect::VideoChromaSpill { range, .. }, EffectParameter::Range) => Some(range),
            _ => None,
        }
    };
}

impl Effect {
    pub const fn kind(&self) -> EffectKind {
        match self {
            Self::VideoColorAdjust { .. } => EffectKind::VideoColorAdjust,
            Self::VideoBlur { .. } => EffectKind::VideoBlur,
            Self::VideoSharpen { .. } => EffectKind::VideoSharpen,
            Self::VideoVignette { .. } => EffectKind::VideoVignette,
            Self::VideoGrain { .. } => EffectKind::VideoGrain,
            Self::VideoChromaKey { .. } => EffectKind::VideoChromaKey,
            Self::VideoLumaKey { .. } => EffectKind::VideoLumaKey,
            Self::VideoChromaSpill { .. } => EffectKind::VideoChromaSpill,
            Self::VideoStabilize { .. } => EffectKind::VideoStabilize,
            Self::AudioNormalize { .. } => EffectKind::AudioNormalize,
            Self::VideoPluginReferenceMonochromeV1 { .. } => {
                EffectKind::VideoPluginReferenceMonochromeV1
            }
        }
    }

    pub const fn domain(&self) -> EffectDomain {
        match self {
            Self::AudioNormalize { .. } => EffectDomain::Audio,
            _ => EffectDomain::Video,
        }
    }

    pub fn curve(&self, parameter: EffectParameter) -> Option<&Animatable<f64>> {
        curve_ref!(self, parameter)
    }

    pub fn curve_mut(&mut self, parameter: EffectParameter) -> Option<&mut Animatable<f64>> {
        curve_ref!(self, parameter)
    }

    pub fn color(&self, parameter: EffectParameter) -> Option<&Color> {
        match (self, parameter) {
            (Self::VideoChromaKey { color, .. }, EffectParameter::Color)
            | (Self::VideoChromaSpill { color, .. }, EffectParameter::Color) => Some(color),
            _ => None,
        }
    }

    pub fn boolean(&self, parameter: EffectParameter) -> Option<bool> {
        match (self, parameter) {
            (Self::VideoLumaKey { invert, .. }, EffectParameter::Invert) => Some(*invert),
            (Self::VideoStabilize { enabled }, EffectParameter::Enabled) => Some(*enabled),
            _ => None,
        }
    }

    pub fn number(&self, parameter: EffectParameter) -> Option<f64> {
        match (self, parameter) {
            (Self::AudioNormalize { target_lufs }, EffectParameter::TargetLufs) => {
                Some(*target_lufs)
            }
            _ => None,
        }
    }

    pub fn parameter(&self, parameter: EffectParameter) -> Option<EffectParameterRef<'_>> {
        if let Some(value) = self.curve(parameter) {
            return Some(EffectParameterRef::Curve(value));
        }
        if let Some(value) = self.color(parameter) {
            return Some(EffectParameterRef::Color(*value));
        }
        if let Some(value) = self.boolean(parameter) {
            return Some(EffectParameterRef::Boolean(value));
        }
        self.number(parameter).map(EffectParameterRef::Number)
    }

    pub fn set_parameter(
        &mut self,
        parameter: EffectParameter,
        value: EffectParameterValue,
    ) -> Option<bool> {
        match value {
            EffectParameterValue::Curve(value) => {
                self.curve_mut(parameter).map(|slot| replace(slot, value))
            }
            EffectParameterValue::Color(value) => set_color(self, parameter, value),
            EffectParameterValue::Boolean(value) => set_boolean(self, parameter, value),
            EffectParameterValue::Number(value) => set_number(self, parameter, value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EffectParameterRef<'a> {
    Curve(&'a Animatable<f64>),
    Number(f64),
    Boolean(bool),
    Color(Color),
}

fn set_color(effect: &mut Effect, parameter: EffectParameter, value: Color) -> Option<bool> {
    match (effect, parameter) {
        (Effect::VideoChromaKey { color, .. }, EffectParameter::Color)
        | (Effect::VideoChromaSpill { color, .. }, EffectParameter::Color) => {
            Some(replace(color, value))
        }
        _ => None,
    }
}

fn set_boolean(effect: &mut Effect, parameter: EffectParameter, value: bool) -> Option<bool> {
    match (effect, parameter) {
        (Effect::VideoLumaKey { invert, .. }, EffectParameter::Invert) => {
            Some(replace(invert, value))
        }
        (Effect::VideoStabilize { enabled }, EffectParameter::Enabled) => {
            Some(replace(enabled, value))
        }
        _ => None,
    }
}

fn set_number(effect: &mut Effect, parameter: EffectParameter, value: f64) -> Option<bool> {
    match (effect, parameter) {
        (Effect::AudioNormalize { target_lufs }, EffectParameter::TargetLufs) => {
            Some(replace(target_lufs, value))
        }
        _ => None,
    }
}

fn replace<T: PartialEq>(slot: &mut T, value: T) -> bool {
    if *slot == value {
        false
    } else {
        *slot = value;
        true
    }
}
