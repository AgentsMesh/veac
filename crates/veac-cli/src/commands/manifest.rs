use std::path::Path;

use veac_artifact::{BuildManifest, ToolFingerprint};

use crate::environment::Environment;
use crate::error::{CliError, CliResult};

pub(crate) fn run(
    project: &Path,
    config: Option<&str>,
    bindings: Option<&Path>,
    output: Option<&Path>,
    environment: &dyn Environment,
) -> CliResult {
    let prepared = crate::planning::prepare_with_bindings(project, config, bindings, environment)?;
    let fingerprint = environment.ffmpeg_fingerprint()?;
    let tools = vec![ToolFingerprint {
        name: "ffmpeg".to_owned(),
        version: fingerprint.version,
        configuration: fingerprint.configuration,
    }];
    let manifest = artifact_result(BuildManifest::from_plan(&prepared.plan, tools, vec![]))?;
    let bytes = artifact_result(veac_artifact::canonical_manifest_bytes(&manifest))?;
    let mut json = String::from_utf8_lossy(&bytes).into_owned();
    json.push('\n');
    match output {
        Some(path) => {
            let protected = std::iter::once(prepared.project_file.as_path())
                .chain(
                    prepared
                        .material_paths
                        .values()
                        .map(std::path::PathBuf::as_path),
                )
                .chain(prepared.binding_file.as_deref());
            let output = crate::output::guarded_write_many(path, protected)?;
            crate::fs::atomic_write(&output, &json)
        }
        None => crate::fs::write_stdout(&json),
    }
}

fn artifact_result<T, E: std::fmt::Display>(value: Result<T, E>) -> CliResult<T> {
    match value {
        Ok(value) => Ok(value),
        Err(error) => Err(CliError::new("MANIFEST_FAILED", error.to_string())),
    }
}
