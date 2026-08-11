use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Animatable, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    VideoColorAdjust,
    VideoBlur,
    VideoDirectionalBlur,
    VideoSharpen,
    VideoVignette,
    VideoGrain,
    VideoChromaKey,
    VideoLumaKey,
    VideoChromaSpill,
    VideoStabilize,
    AudioNormalize,
    VideoPluginReferenceMonochromeV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectDomain {
    Video,
    Audio,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EffectParameter {
    Brightness,
    Contrast,
    Saturation,
    AngleDegrees,
    Radius,
    Amount,
    Color,
    Similarity,
    Blend,
    Threshold,
    Tolerance,
    Softness,
    Invert,
    Range,
    Enabled,
    TargetLufs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum EffectParameterValue {
    Curve(Animatable<f64>),
    Number(f64),
    Boolean(bool),
    Color(Color),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct PluginEffectDigest(#[schemars(regex(pattern = r"^[0-9a-f]{64}$"))] String);

impl PluginEffectDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, PluginEffectDigestError> {
        let value = value.into();
        if value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            Ok(Self(value))
        } else {
            Err(PluginEffectDigestError(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_valid(&self) -> bool {
        Self::new(self.0.clone()).is_ok()
    }
}

impl fmt::Display for PluginEffectDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginEffectDigestError(String);

impl fmt::Display for PluginEffectDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid plugin effect descriptor digest {:?}",
            self.0
        )
    }
}

impl std::error::Error for PluginEffectDigestError {}
