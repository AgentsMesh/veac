use std::path::Path;

use veac_artifact::ArtifactStore;
use veac_runtime::executor::BundleExecutor;
use veac_runtime::{RuntimeError, RuntimeErrorKind};

use crate::arguments::SubstitutionPolicy;
use crate::environment::Environment;
use crate::error::{CliError, CliResult};

mod adapter;
mod substitution;
#[cfg(test)]
#[path = "render/tests.rs"]
mod tests;

pub(crate) fn run(
    project: &Path,
    config: Option<&str>,
    input_resolution: crate::planning::InputResolution<'_>,
    destination: Option<&Path>,
    proxy_policy: SubstitutionPolicy,
    segment_policy: SubstitutionPolicy,
    environment: &dyn Environment,
) -> CliResult {
    let mut prepared = crate::planning::prepare_with_input_resolution(
        project,
        config,
        input_resolution,
        environment,
    )?;
    let outputs = crate::output::bind_render_outputs(&mut prepared, destination)?;
    let project_directory = prepared.project_file.parent().unwrap_or(Path::new("."));
    let store = ArtifactStore::new(project_directory.join(".veac-artifacts"));
    let needs_fingerprint = proxy_policy != SubstitutionPolicy::Original
        || segment_policy != SubstitutionPolicy::Original;
    let fingerprint = needs_fingerprint
        .then(|| environment.ffmpeg_fingerprint())
        .transpose()?;
    let disposition = substitution::prepare(
        &mut prepared,
        &store,
        proxy_policy,
        segment_policy,
        fingerprint,
        environment,
    )?;
    let bundle = veac_codegen::emitter::emit_all(&prepared.plan, &prepared.bindings)
        .map_err(crate::diagnostic::codegen)?;
    println!(
        "Rendering {} -> {} deliverable(s)",
        prepared.plan.output.render_config_id,
        outputs.len()
    );
    let execution = BundleExecutor::new(adapter::RuntimeEnvironment(environment))
        .execute(&bundle, &store)
        .map_err(runtime_error)?;
    substitution::store(&prepared, &store, disposition, &execution, environment)?;
    let hits = execution.tasks.iter().filter(|task| task.cache_hit).count();
    for path in outputs {
        println!("Rendered: {}", path.display());
    }
    println!("Checkpoints: {hits}/{} reused", execution.tasks.len());
    Ok(())
}

fn runtime_error(error: RuntimeError) -> CliError {
    if error.kind == RuntimeErrorKind::ResourceLimit {
        CliError::resource_limit("RENDER_FAILED", error.to_string())
    } else {
        CliError::new("RENDER_FAILED", error.to_string())
    }
}
