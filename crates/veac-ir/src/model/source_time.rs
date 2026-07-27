use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Rational, RationalTime};

/// Maximum explicit loop fan-out accepted by the canonical editing contract.
pub const MAX_SOURCE_REPEAT: u32 = 1_024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceMapping {
    pub time_map: SourceTimeMap,
    pub frame_synthesis: FrameSynthesisPolicy,
    pub out_of_range: SourceOutOfRangePolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SourceOutOfRangePolicy {
    Strict,
    HoldFirst,
    HoldLast,
    HoldBoth,
}

impl SourceOutOfRangePolicy {
    pub fn allows_before(self) -> bool {
        matches!(self, Self::HoldFirst | Self::HoldBoth)
    }

    pub fn allows_after(self) -> bool {
        matches!(self, Self::HoldLast | Self::HoldBoth)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceTimeMap {
    Linear {
        source_start: RationalTime,
        rate: Rational,
        repeat: u32,
        direction: PlaybackDirection,
    },
    Curve {
        segments: Vec<SourceTimeSegment>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceTimeSegment {
    /// Duration on the clip's record timeline. Segments are contiguous in vector order.
    pub record_duration: RationalTime,
    pub source_start: RationalTime,
    pub source_end: RationalTime,
    pub interpolation: SourceTimeInterpolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceTimeInterpolation {
    Linear,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackDirection {
    Forward,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FrameSynthesisPolicy {
    Nearest,
    Blend,
    MotionCompensated,
}

impl SourceMapping {
    pub fn linear(source_start: RationalTime, rate: Rational) -> Self {
        Self {
            time_map: SourceTimeMap::Linear {
                source_start,
                rate,
                repeat: 1,
                direction: PlaybackDirection::Forward,
            },
            frame_synthesis: FrameSynthesisPolicy::Nearest,
            out_of_range: SourceOutOfRangePolicy::Strict,
        }
    }
}
