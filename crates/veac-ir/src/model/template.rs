use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::RationalTime;

pub const SLOT_LABEL_MAX_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SlotConstraint {
    pub kind: SlotKind,
    pub fill: FillMode,
    pub label: String,
    pub min_source_duration: Option<RationalTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SlotKind {
    Video,
    Image,
    VideoOrImage,
}

impl SlotKind {
    pub fn accepts(self, kind: crate::MaterialKind) -> bool {
        match self {
            Self::Video => kind == crate::MaterialKind::Video,
            Self::Image => kind == crate::MaterialKind::Image,
            Self::VideoOrImage => matches!(
                kind,
                crate::MaterialKind::Video | crate::MaterialKind::Image
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FillMode {
    FitDuration,
    TakeHead,
    TakeCenter,
}
