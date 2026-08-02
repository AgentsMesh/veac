use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};

pub(super) fn run(
    project: &Path,
    sequence: Option<&str>,
    output: Option<&Path>,
    loss_path: Option<&Path>,
    allow_lossy: bool,
) -> CliResult {
    let loaded = crate::canonical::load_local(project)?;
    let sequence_id = match sequence {
        Some(value) => veac_ir::SequenceId::new(value)
            .map_err(|error| CliError::new("OTIO_SEQUENCE_ID", error.to_string()))?,
        None => loaded.envelope.project.entry_sequence_id.clone(),
    };
    let exported =
        veac_otio::export_sequence(&loaded.envelope, &sequence_id).map_err(super::io::error)?;
    super::io::require_loss(&exported.loss_report, allow_lossy, loss_path)?;
    let timeline = veac_otio::canonical_otio_json(&exported.timeline).map_err(super::io::error)?;
    let loss = loss_json(&exported.loss_report, loss_path)?;
    let protected = protected(&loaded);
    super::io::write_outputs(
        &timeline,
        output,
        loss.as_ref().map(|(json, path)| (json.as_str(), *path)),
        &protected,
    )
}

fn loss_json<'a>(
    report: &veac_otio::OtioLossReport,
    path: Option<&'a Path>,
) -> CliResult<Option<(String, &'a Path)>> {
    path.map(|path| {
        veac_otio::canonical_otio_loss_json(report)
            .map(|json| (json, path))
            .map_err(super::io::error)
    })
    .transpose()
}

pub(super) fn protected(loaded: &crate::canonical::LoadedProject) -> Vec<PathBuf> {
    let mut paths = vec![loaded.project_file.clone()];
    paths.extend(crate::canonical::local_material_paths(
        &loaded.envelope,
        &loaded.project_file,
    ));
    paths
}
