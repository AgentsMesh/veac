use std::path::Path;

use crate::environment::Environment;
use crate::error::{CliError, CliResult};
use crate::PlanFormat;

pub(crate) fn run(
    project: &Path,
    config: Option<&str>,
    bindings: Option<&Path>,
    material_root: Option<&Path>,
    format: PlanFormat,
    environment: &dyn Environment,
) -> CliResult {
    let prepared = crate::planning::prepare_with_input_resolution(
        project,
        config,
        crate::planning::InputResolution::new(material_root, bindings),
        environment,
    )?;
    let rendered = match format {
        PlanFormat::Json => veac_plan::canonical_plan_json(&prepared.plan),
    };
    let mut rendered = match rendered {
        Ok(rendered) => rendered,
        Err(error) => return Err(CliError::new("PLAN_ENCODE", error.to_string())),
    };
    rendered.push('\n');
    crate::fs::write_stdout(&rendered)
}
