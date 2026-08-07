use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{TemporalInputId, TemporalParameterId, TemporalType};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TemporalClock {
    SequenceTime,
    ClipTime,
    SourceTime,
    Frame,
    Progress,
}

impl TemporalClock {
    pub const fn value_type(self) -> TemporalType {
        match self {
            Self::SequenceTime | Self::ClipTime | Self::SourceTime => TemporalType::Time,
            Self::Frame => TemporalType::Integer,
            Self::Progress => TemporalType::Scalar,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TemporalInputSource {
    Clock { clock: TemporalClock },
    Parameter { parameter_id: TemporalParameterId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalInputDeclaration {
    pub id: TemporalInputId,
    pub value_type: TemporalType,
    pub source: TemporalInputSource,
}
