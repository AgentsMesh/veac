mod ass;
mod common;
mod ids;
pub(crate) mod native;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CaptionDocument, CaptionError, CaptionFormat, LossReport, OverlapPolicy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportOptions {
    pub timescale: u32,
    pub overlap_policy: OverlapPolicy,
    pub id_namespace: String,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            timescale: 1000,
            overlap_policy: OverlapPolicy::Reject,
            id_namespace: "caption".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportResult {
    pub document: CaptionDocument,
    pub loss_report: LossReport,
}

pub fn import_caption(
    input: &str,
    format: CaptionFormat,
    options: &ImportOptions,
) -> Result<ImportResult, CaptionError> {
    if input.trim().is_empty() {
        return Err(CaptionError::parse(format, "input is empty"));
    }
    if options.id_namespace.trim().is_empty() {
        return Err(CaptionError::parse(format, "ID namespace is empty"));
    }
    match format {
        CaptionFormat::Srt => common::import_srt(input, options),
        CaptionFormat::WebVtt => common::import_vtt(input, options),
        CaptionFormat::Ass => ass::import_ass(input, options),
    }
}
