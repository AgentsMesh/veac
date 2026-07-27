use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtioLossReport {
    pub losses: Vec<OtioLoss>,
}

impl OtioLossReport {
    pub fn is_empty(&self) -> bool {
        self.losses.is_empty()
    }

    pub fn len(&self) -> usize {
        self.losses.len()
    }

    pub(crate) fn push(
        &mut self,
        pointer: impl Into<String>,
        field: impl Into<String>,
        reason: impl Into<String>,
        preserved_in_extension: bool,
    ) {
        self.losses.push(OtioLoss {
            pointer: pointer.into(),
            field: field.into(),
            reason: reason.into(),
            preserved_in_extension,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtioLoss {
    pub pointer: String,
    pub field: String,
    pub reason: String,
    pub preserved_in_extension: bool,
}
