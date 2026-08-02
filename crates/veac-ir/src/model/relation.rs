use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ApplyId, BusId, ItemId, RelationId, SequenceId, SidechainSource, TimeRange, TrackId,
    TrackMatteMode, Transition,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub id: RelationId,
    pub sequence_id: SequenceId,
    pub kind: RelationKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RelationKind {
    Transition {
        from: RelationEndpoint,
        to: RelationEndpoint,
        transition: Transition,
    },
    Matte {
        producer: RelationEndpoint,
        consumer: RelationEndpoint,
        parameters: MatteRelationParameters,
    },
    Sidechain {
        key: RelationEndpoint,
        target: RelationEndpoint,
        parameters: SidechainRelationParameters,
    },
    Group {
        members: Vec<RelationEndpoint>,
    },
    AvLink {
        video: RelationEndpoint,
        audio: Vec<RelationEndpoint>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatteRelationParameters {
    pub mode: TrackMatteMode,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RelationEndpoint {
    Item { item_id: ItemId },
    Apply { apply_id: ApplyId },
    Track { track_id: TrackId },
    Bus { bus_id: BusId },
}

impl RelationEndpoint {
    pub fn item(item_id: ItemId) -> Self {
        Self::Item { item_id }
    }

    pub fn track(track_id: TrackId) -> Self {
        Self::Track { track_id }
    }

    pub fn apply(apply_id: ApplyId) -> Self {
        Self::Apply { apply_id }
    }

    pub fn bus(bus_id: BusId) -> Self {
        Self::Bus { bus_id }
    }

    pub fn item_id(&self) -> Option<&ItemId> {
        match self {
            Self::Item { item_id } => Some(item_id),
            _ => None,
        }
    }

    pub fn apply_id(&self) -> Option<&ApplyId> {
        match self {
            Self::Apply { apply_id } => Some(apply_id),
            _ => None,
        }
    }

    pub fn sidechain_source(&self) -> Option<SidechainSource> {
        match self {
            Self::Track { track_id } => Some(SidechainSource::Track {
                track_id: track_id.clone(),
            }),
            Self::Bus { bus_id } => Some(SidechainSource::Bus {
                bus_id: bus_id.clone(),
            }),
            Self::Item { .. } | Self::Apply { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SidechainRelationParameters {
    pub threshold_db: f64,
    pub ratio: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
    /// Half-open range relative to the target item's record start, not sequence time.
    pub active_range: Option<TimeRange>,
}
