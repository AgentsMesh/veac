use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::CaptionCueId;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LossReport {
    pub losses: Vec<CaptionLoss>,
}

impl LossReport {
    pub fn is_empty(&self) -> bool {
        self.losses.is_empty()
    }

    pub(crate) fn cue(&mut self, id: &CaptionCueId, field: &str, reason: &str) {
        self.losses.push(CaptionLoss {
            cue_id: Some(id.clone()),
            field: field.to_owned(),
            reason: reason.to_owned(),
        });
    }

    pub(crate) fn document(&mut self, field: &str, reason: &str) {
        self.losses.push(CaptionLoss {
            cue_id: None,
            field: field.to_owned(),
            reason: reason.to_owned(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionLoss {
    pub cue_id: Option<CaptionCueId>,
    pub field: String,
    pub reason: String,
}
