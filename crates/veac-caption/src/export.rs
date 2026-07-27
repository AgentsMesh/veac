mod common;
mod losses;
mod markup;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CaptionEnvelope, CaptionError, CaptionFormat, LossReport};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportResult {
    pub content: String,
    pub loss_report: LossReport,
}

pub fn export_caption(
    value: &CaptionEnvelope,
    format: CaptionFormat,
) -> Result<ExportResult, CaptionError> {
    crate::validate(value)?;
    common::export(value, format)
}
