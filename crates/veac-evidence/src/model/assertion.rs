use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{AlphaExpectation, BoundsExpectation, DiffExpectation};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AssertionSpec {
    DecodeComplete(DecodeCompleteSpec),
    Alpha(AlphaSpec),
    PixelDiff(PixelDiffSpec),
    Bounds(BoundsSpec),
    LayerOrder(LayerOrderSpec),
    CompositeOver(CompositeOverSpec),
    RevealOrder(RevealOrderSpec),
    MotionProfile(MotionProfileSpec),
}

impl AssertionSpec {
    pub fn id(&self) -> &str {
        match self {
            Self::DecodeComplete(value) => &value.id,
            Self::Alpha(value) => &value.id,
            Self::PixelDiff(value) => &value.id,
            Self::Bounds(value) => &value.id,
            Self::LayerOrder(value) => &value.id,
            Self::CompositeOver(value) => &value.id,
            Self::RevealOrder(value) => &value.id,
            Self::MotionProfile(value) => &value.id,
        }
    }

    pub fn kind(&self) -> AssertionKind {
        match self {
            Self::DecodeComplete(_) => AssertionKind::DecodeComplete,
            Self::Alpha(_) => AssertionKind::Alpha,
            Self::PixelDiff(_) => AssertionKind::PixelDiff,
            Self::Bounds(_) => AssertionKind::Bounds,
            Self::LayerOrder(_) => AssertionKind::LayerOrder,
            Self::CompositeOver(_) => AssertionKind::CompositeOver,
            Self::RevealOrder(_) => AssertionKind::RevealOrder,
            Self::MotionProfile(_) => AssertionKind::MotionProfile,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssertionKind {
    DecodeComplete,
    Alpha,
    PixelDiff,
    Bounds,
    LayerOrder,
    CompositeOver,
    RevealOrder,
    MotionProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecodeCompleteSpec {
    pub id: String,
    pub source_id: String,
    pub minimum_frames: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlphaSpec {
    pub id: String,
    pub sample_id: String,
    pub region_id: Option<String>,
    pub expectation: AlphaExpectation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiffChannels {
    Rgb,
    Rgba,
    Alpha,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PixelDiffSpec {
    pub id: String,
    pub left_sample_id: String,
    pub right_sample_id: String,
    pub region_id: Option<String>,
    pub channels: DiffChannels,
    pub change_threshold: u8,
    pub expectation: DiffExpectation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BoundsSpec {
    pub id: String,
    pub sample_id: String,
    pub region_id: Option<String>,
    pub mask: MaskSpec,
    pub expectation: BoundsExpectation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MaskSpec {
    Alpha {
        minimum: u8,
    },
    Difference {
        reference_sample_id: String,
        minimum_delta: u8,
    },
    Luma {
        threshold: u8,
        above: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LayerOrderSpec {
    pub id: String,
    pub upper_entity: String,
    pub lower_entity: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompositeOverSpec {
    pub id: String,
    pub actual_sample_id: String,
    pub underlay_sample_id: String,
    pub overlay_sample_id: String,
    pub region_id: Option<String>,
    pub overlay_alpha_minimum: u8,
    pub maximum_expected_rmse: f64,
    pub minimum_improvement_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RevealOrderSpec {
    pub id: String,
    pub baseline_sample_id: String,
    pub subjects: Vec<RevealSubject>,
    pub checkpoints: Vec<RevealCheckpoint>,
    pub change_threshold: u8,
    pub visible_minimum: f64,
    pub hidden_maximum: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RevealSubject {
    pub id: String,
    pub region_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RevealCheckpoint {
    pub sample_id: String,
    pub visible_prefix: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MotionProfileSpec {
    pub id: String,
    pub sample_ids: Vec<String>,
    pub region_id: Option<String>,
    pub metric: MotionMetric,
    pub minimum_interval_motion: f64,
    pub minimum_deceleration_ratio: f64,
    pub monotonic_tolerance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum MotionMetric {
    ChangedFraction { threshold: u8 },
    AlphaCentroid { minimum: u8 },
}
