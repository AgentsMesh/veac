mod entry;
mod event;
mod text;

pub use entry::*;
pub use event::*;
pub use text::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntityAuthorship {
    pub logical_path: Vec<LogicalPathSegment>,
    pub events: Vec<AuthorshipEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectAuthorship {
    pub entity: EntityAuthorship,
    pub multicam_groups: Vec<MulticamAuthorship>,
    pub annotations: Vec<AnnotationAuthorship>,
    pub deliveries: Vec<DeliveryAuthorship>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SequenceAuthorship {
    Veac {
        entity: EntityAuthorship,
        tracks: Vec<TrackAuthorship>,
        relations: Vec<RelationAuthorship>,
        applies: Vec<ApplyAuthorship>,
    },
    Otio {
        document_sha256: Sha256Digest,
    },
}
