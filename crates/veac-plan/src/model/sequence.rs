use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{PlacementMode, RationalTime, SequenceId, SequenceSettings, TrackId, TrackKind};

use super::{ResolvedApply, ResolvedClip, ResolvedTransition};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSequence {
    pub id: SequenceId,
    pub name: String,
    pub settings: SequenceSettings,
    pub duration: RationalTime,
    /// Tracks sorted by `(order, source_order, id)`.
    pub tracks: Vec<ResolvedTrack>,
    /// Enabled applications retain canonical vector order through `source_order`.
    pub applies: Vec<ResolvedApply>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTrack {
    pub id: TrackId,
    /// Canonical vector position, retained when authored track orders are equal.
    pub source_order: u32,
    pub kind: TrackKind,
    pub order: i32,
    pub placement_mode: PlacementMode,
    pub state: EffectiveTrackState,
    pub routing: ResolvedTrackRouting,
    /// Enabled clips sorted by `(record_start, source_order, id)`.
    pub clips: Vec<ResolvedClip>,
    pub transitions: Vec<ResolvedTransition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveTrackState {
    pub include_in_render: bool,
    pub visual_enabled: bool,
    pub audio_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTrackRouting {
    pub visual: Option<ResolvedVisualRoute>,
    pub audio: Option<ResolvedAudioRoute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedVisualRoute {
    MainComposite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedAudioRoute {
    MainMix,
    Bus { bus_id: String },
}
