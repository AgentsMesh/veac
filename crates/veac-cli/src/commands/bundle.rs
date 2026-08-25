use std::path::Path;

use crate::environment::Environment;
use crate::error::{CliError, CliResult};

pub(crate) fn run(
    project: &Path,
    config: Option<&str>,
    bindings: Option<&Path>,
    material_root: Option<&Path>,
    destination: &Path,
    environment: &dyn Environment,
) -> CliResult {
    let prepared = crate::planning::prepare_with_input_resolution(
        project,
        config,
        crate::planning::InputResolution::new(material_root, bindings),
        environment,
    )?;
    let mut protected = vec![prepared.project_file.clone()];
    protected.extend(prepared.binding_file.iter().cloned());
    protected.extend(prepared.material_paths.values().cloned());
    if let Some(root) = &prepared.material_root {
        protected.extend(root.local_paths(&prepared.project));
    }
    let destination = crate::output::guarded_package_directory(
        destination,
        protected.iter().map(std::path::PathBuf::as_path),
    )?;
    veac_artifact::package_plan(
        &prepared.project,
        &prepared.plan,
        &prepared.bindings,
        &destination,
    )
    .map_err(|error| CliError::new("BUNDLE_FAILED", error.to_string()))?;
    println!("Bundled: {}", destination.display());
    Ok(())
}
