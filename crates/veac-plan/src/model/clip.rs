use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    CaptionCueSemantics, Generator, ItemId, RationalTime, SequenceId, StreamSelection, TimeRange,
};

use super::{
    EffectiveAudioProperties, EffectiveVisualProperties, PlanInputId, ResolvedEffect,
    ResolvedMulticamSource, ResolvedSourceMapping, ResolvedText,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedClip {
    pub id: ItemId,
    /// Canonical vector position, retained as the final semantic tie-breaker for equal starts.
    pub source_order: u32,
    pub record_range: TimeRange,
    pub source: ResolvedClipSource,
    pub source_mapping: Option<ResolvedSourceMapping>,
    pub visual: Option<EffectiveVisualProperties>,
    pub audio: Option<EffectiveAudioProperties>,
    pub effects: Vec<ResolvedEffect>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedClipSource {
    Media {
        input_id: PlanInputId,
        video_stream: Option<StreamSelection>,
        audio_stream: Option<StreamSelection>,
    },
    FreezeFrame {
        input_id: PlanInputId,
        video_stream: StreamSelection,
        source_time: RationalTime,
    },
    Sequence {
        sequence_id: SequenceId,
    },
    Multicam {
        source: ResolvedMulticamSource,
    },
    Text {
        content: ResolvedText,
    },
    Caption {
        content: ResolvedText,
        speaker: Option<String>,
        cue: CaptionCueSemantics,
    },
    Generated {
        generator: Generator,
    },
}
