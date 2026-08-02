use std::path::Path;

use crate::environment::Environment;
use crate::error::{CliError, CliResult};

pub(crate) fn run(
    project: &Path,
    config: Option<&str>,
    bindings: Option<&Path>,
    destination: &Path,
    environment: &dyn Environment,
) -> CliResult {
    let prepared = crate::planning::prepare_with_bindings(project, config, bindings, environment)?;
    veac_artifact::package_plan(
        &prepared.project,
        &prepared.plan,
        &prepared.bindings,
        destination,
    )
    .map_err(|error| CliError::new("PACKAGE_FAILED", error.to_string()))?;
    println!("Packaged: {}", destination.display());
    Ok(())
}
