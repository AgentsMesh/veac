use std::path::Path;

use veac_caption::{CaptionEnvelope, CaptionFormat, ImportOptions, OverlapPolicy};

use crate::error::{CliError, CliResult};

pub(super) struct ImportRequest<'a> {
    pub input: &'a Path,
    pub format: CaptionFormat,
    pub timescale: u32,
    pub overlap_policy: OverlapPolicy,
    pub namespace: &'a str,
    pub output: Option<&'a Path>,
    pub loss_path: Option<&'a Path>,
    pub allow_lossy: bool,
}

pub(super) fn import(request: ImportRequest<'_>) -> CliResult {
    let input = crate::fs::canonical_file(request.input, "caption sidecar")?;
    let content = crate::fs::read_utf8(&input, "caption sidecar")?;
    let result = veac_caption::import_caption(
        &content,
        request.format,
        &ImportOptions {
            timescale: request.timescale,
            overlap_policy: request.overlap_policy,
            id_namespace: request.namespace.to_owned(),
        },
    )
    .map_err(super::io::caption_error)?;
    super::io::require_acknowledged_loss(&result.loss_report, request.allow_lossy)?;
    let document = veac_caption::canonical_caption_json(&CaptionEnvelope::new(result.document))
        .map_err(super::io::caption_error)?;
    let loss = loss_json(&result.loss_report, request.loss_path)?;
    super::io::write_outputs(
        &document,
        request.output,
        loss.as_ref().map(|(json, path)| (json.as_str(), *path)),
        &[input],
    )
}

pub(super) fn export(
    document: &Path,
    format: CaptionFormat,
    output: Option<&Path>,
    loss_path: Option<&Path>,
    allow_lossy: bool,
) -> CliResult {
    let document = crate::fs::canonical_file(document, "caption document")?;
    let input = crate::fs::read_utf8(&document, "caption document")?;
    let envelope = veac_caption::decode_caption_json(&input).map_err(super::io::caption_error)?;
    let result =
        veac_caption::export_caption(&envelope, format).map_err(super::io::caption_error)?;
    super::io::require_acknowledged_loss(&result.loss_report, allow_lossy)?;
    let loss = loss_json(&result.loss_report, loss_path)?;
    super::io::write_outputs(
        &result.content,
        output,
        loss.as_ref().map(|(json, path)| (json.as_str(), *path)),
        &[document],
    )
}

fn loss_json<'a>(
    report: &veac_caption::LossReport,
    path: Option<&'a Path>,
) -> CliResult<Option<(String, &'a Path)>> {
    path.map(|path| {
        veac_caption::canonical_loss_report_json(report)
            .map(|json| (json, path))
            .map_err(super::io::caption_error)
    })
    .transpose()
}

pub(super) fn invalid_id(error: impl std::fmt::Display) -> CliError {
    CliError::new("CAPTION_BINDING_ID", error.to_string())
}
