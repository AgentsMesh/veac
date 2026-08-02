use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    FrameSynthesisPolicy, PlaybackDirection, Rational, SourceOutOfRangePolicy, SourceTimeSegment,
    TimeRange,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSourceMapping {
    pub time_map: ResolvedSourceTimeMap,
    pub frame_synthesis: FrameSynthesisPolicy,
    pub out_of_range: SourceOutOfRangePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedSourceTimeMap {
    Linear {
        /// Exact source interval consumed by one repeat.
        source_range_per_repeat: TimeRange,
        rate: Rational,
        repeat: u32,
        direction: PlaybackDirection,
    },
    Curve {
        segments: Vec<SourceTimeSegment>,
    },
}
