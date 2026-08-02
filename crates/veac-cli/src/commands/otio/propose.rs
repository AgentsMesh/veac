use std::path::Path;

use crate::error::{CliError, CliResult};

pub(super) struct Request<'a> {
    pub project: &'a Path,
    pub timeline: &'a Path,
    pub bindings: Option<&'a Path>,
    pub operation_id: &'a str,
    pub output: Option<&'a Path>,
    pub loss_report: Option<&'a Path>,
    pub allow_lossy: bool,
}

pub(super) fn run(request: Request<'_>) -> CliResult {
    let loaded = crate::canonical::load_local(request.project)?;
    let timeline_file = crate::fs::canonical_file(request.timeline, "OTIO timeline")?;
    let input = crate::fs::read_utf8(&timeline_file, "OTIO timeline")?;
    let timeline = veac_otio::decode_otio_json(&input).map_err(super::io::error)?;
    let (imported, binding_file) = match request.bindings {
        Some(path) => {
            let path = crate::fs::canonical_file(path, "OTIO import bindings")?;
            let bindings = super::io::read_json(&path, "OTIO import bindings")?;
            let imported =
                veac_otio::import_bound_timeline(&timeline, &bindings).map_err(super::io::error)?;
            (imported, Some(path))
        }
        None => (
            veac_otio::import_veac_extension(&timeline).map_err(super::io::error)?,
            None,
        ),
    };
    super::io::require_loss(
        &imported.loss_report,
        request.allow_lossy,
        request.loss_report,
    )?;
    let operation_id = veac_ir::OperationId::new(request.operation_id)
        .map_err(|error| CliError::new("OTIO_OPERATION_ID", error.to_string()))?;
    let proposal = veac_otio::propose_import(
        &loaded.envelope,
        &imported,
        operation_id,
        request.allow_lossy,
    )
    .map_err(super::io::error)?;
    let json = veac_otio::canonical_otio_proposal_json(&proposal).map_err(super::io::error)?;
    let loss = request
        .loss_report
        .map(|path| {
            veac_otio::canonical_otio_loss_json(&imported.loss_report)
                .map(|json| (json, path))
                .map_err(super::io::error)
        })
        .transpose()?;
    let mut protected = super::export::protected(&loaded);
    protected.push(timeline_file);
    protected.extend(binding_file);
    super::io::write_outputs(
        &json,
        request.output,
        loss.as_ref().map(|(json, path)| (json.as_str(), *path)),
        &protected,
    )
}
