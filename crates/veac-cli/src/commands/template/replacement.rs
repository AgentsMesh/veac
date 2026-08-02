use std::path::{Path, PathBuf};

use veac_ir::MaterialSource;

use crate::environment::Environment;
use crate::{CliError, CliResult};

pub(super) fn verify(
    request: &veac_template::TemplateFillRequest,
    project_file: &Path,
    environment: &dyn Environment,
) -> CliResult<Vec<PathBuf>> {
    let base = project_file.parent().unwrap_or(Path::new("."));
    let mut paths = Vec::with_capacity(request.media_bindings.len());
    for binding in &request.media_bindings {
        let uri = match &binding.material.source {
            MaterialSource::File { uri } if veac_ir::project_relative_uri_valid(uri) => uri,
            MaterialSource::File { .. } => {
                return Err(CliError::new(
                    "TEMPLATE_REPLACEMENT_URI_INVALID",
                    format!(
                        "template replacement for {} is not project-relative",
                        binding.clip_id
                    ),
                ))
            }
            MaterialSource::Remote { uri } => {
                return Err(CliError::new(
                    "TEMPLATE_REPLACEMENT_UNMATERIALIZED",
                    format!(
                        "template replacement for {} is not materialized: {uri}",
                        binding.clip_id
                    ),
                ))
            }
        };
        let path = crate::fs::canonical_file(&base.join(uri), "template replacement material")?;
        let observed = environment.probe(&path, binding.material.stream_intent.clone())?;
        if binding.material.identity.as_ref() != Some(&observed.observed_identity) {
            return Err(CliError::new(
                "TEMPLATE_REPLACEMENT_IDENTITY_MISMATCH",
                format!(
                    "template replacement bytes do not match the authored identity for {}",
                    binding.clip_id
                ),
            ));
        }
        if binding.material.probe.as_ref() != Some(&observed) {
            return Err(CliError::new(
                "TEMPLATE_REPLACEMENT_PROBE_MISMATCH",
                format!(
                    "template replacement probe facts do not match the local media for {}",
                    binding.clip_id
                ),
            ));
        }
        paths.push(path);
    }
    Ok(paths)
}
