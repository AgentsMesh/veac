use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Animatable, Apply, AudioCrossfade, AudioProcessor, BusId, EffectInstance, FontRef, Generator,
    ItemId, MaterialId, MulticamGroupId, MulticamSwitch, Rational, RationalTime, SequenceId,
    SlotConstraint, SourceMapping, TextStyle, TimeRange, TrackId, VisualProperties,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    pub id: SequenceId,
    pub name: String,
    pub settings: SequenceSettings,
    pub tracks: Vec<Track>,
    pub applies: Vec<Apply>,
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SequenceSettings {
    pub width: u32,
    pub height: u32,
    pub frame_rate: Rational,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub id: TrackId,
    pub kind: TrackKind,
    pub order: i32,
    pub placement_mode: PlacementMode,
    pub state: TrackState,
    pub routing: TrackRouting,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrackKind {
    Video,
    Audio,
    Visual,
    Caption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlacementMode {
    Magnetic,
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrackState {
    pub enabled: bool,
    pub muted: bool,
    pub solo: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TrackRouting {
    Default,
    AudioBus { bus_id: BusId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub id: ItemId,
    pub enabled: bool,
    pub record_range: TimeRange,
    pub source: ClipSource,
    pub source_mapping: Option<SourceMapping>,
    pub visual: Option<VisualProperties>,
    pub audio: Option<AudioProperties>,
    pub effects: Vec<EffectInstance>,
    #[serde(deserialize_with = "explicit_option")]
    #[schemars(with = "Option<SlotConstraint>", required)]
    pub replaceable: Option<SlotConstraint>,
    pub template_editable_text: bool,
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClipSource {
    Media {
        material_id: MaterialId,
    },
    FreezeFrame {
        material_id: MaterialId,
        source_time: RationalTime,
    },
    Sequence {
        sequence_id: SequenceId,
    },
    Multicam {
        group_id: MulticamGroupId,
        switches: Vec<MulticamSwitch>,
    },
    Text {
        text: String,
        style: TextStyle,
    },
    Caption {
        text: String,
        speaker: Option<String>,
        style: TextStyle,
    },
    Generated {
        generator: Generator,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioProperties {
    pub gain: Animatable<f64>,
    pub pan: Animatable<f64>,
    pub muted: bool,
    pub normalize: bool,
    pub pitch_policy: PitchPolicy,
    pub processors: Vec<AudioProcessor>,
    pub crossfade: Option<AudioCrossfade>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PitchPolicy {
    Preserve,
    FollowSpeed,
}

impl ClipSource {
    pub fn font_material(&self) -> Option<&MaterialId> {
        let style = match self {
            Self::Text { style, .. } | Self::Caption { style, .. } => style,
            _ => return None,
        };
        match &style.font {
            FontRef::Material { material_id } => Some(material_id),
            FontRef::Family { .. } => None,
        }
    }
}

fn explicit_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}
