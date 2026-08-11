mod access;
mod catalog;
mod defaults;
mod parameter;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Animatable, Color, EffectId, TimeRange};

pub use access::EffectParameterRef;
pub use parameter::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectInstance {
    pub id: EffectId,
    pub enabled: bool,
    pub enable_range: Option<TimeRange>,
    pub effect: Effect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    VideoColorAdjust {
        brightness: Animatable<f64>,
        contrast: Animatable<f64>,
        saturation: Animatable<f64>,
    },
    VideoBlur {
        radius: Animatable<f64>,
    },
    VideoDirectionalBlur {
        angle_degrees: Animatable<f64>,
        radius: Animatable<f64>,
    },
    VideoSharpen {
        amount: Animatable<f64>,
    },
    VideoVignette {
        amount: Animatable<f64>,
    },
    VideoGrain {
        amount: Animatable<f64>,
    },
    VideoChromaKey {
        color: Color,
        similarity: Animatable<f64>,
        blend: Animatable<f64>,
    },
    VideoLumaKey {
        threshold: Animatable<f64>,
        tolerance: Animatable<f64>,
        softness: Animatable<f64>,
        invert: bool,
    },
    VideoChromaSpill {
        color: Color,
        amount: Animatable<f64>,
        range: Animatable<f64>,
    },
    VideoStabilize {
        enabled: bool,
    },
    AudioNormalize {
        target_lufs: f64,
    },
    VideoPluginReferenceMonochromeV1 {
        descriptor_digest: PluginEffectDigest,
        amount: Animatable<f64>,
    },
}

impl EffectInstance {
    pub const fn kind(&self) -> EffectKind {
        self.effect.kind()
    }

    pub const fn domain(&self) -> EffectDomain {
        self.effect.domain()
    }
}
