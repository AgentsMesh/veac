use crate::arguments::RelinkArgs;
use crate::{CliError, CliResult};

pub(crate) fn run(arguments: RelinkArgs) -> CliResult {
    let loaded = crate::canonical::load_local(&arguments.project)?;
    let config = crate::planning::select_config(&loaded.envelope, arguments.config.as_deref())?;
    let plan =
        veac_plan::resolve_one(&loaded.envelope, &config).map_err(crate::diagnostic::resolution)?;
    let candidates = veac_artifact::discover_relink_candidates(&arguments.search)
        .map_err(|error| CliError::new("RELINK_FAILED", error.to_string()))?;
    let resolved = veac_artifact::resolve_relinks(&plan, &candidates)
        .map_err(|error| CliError::new("RELINK_FAILED", error.to_string()))?;
    require_complete(&resolved)?;
    let manifest = veac_artifact::binding_manifest(&plan, &resolved.bindings)
        .map_err(|error| CliError::new("RELINK_FAILED", error.to_string()))?;
    let bytes = veac_artifact::canonical_binding_bytes(&manifest)
        .map_err(|error| CliError::new("RELINK_FAILED", error.to_string()))?;
    let mut protected = vec![loaded.project_file];
    protected.extend(
        resolved
            .bindings
            .inputs()
            .values()
            .filter_map(|binding| binding.resource())
            .map(|resource| resource.path().to_owned()),
    );
    super::workflow_io::write(&bytes, arguments.output.as_deref(), &protected)
}

fn require_complete(resolution: &veac_artifact::RelinkResolution) -> CliResult {
    if resolution.unresolved.is_empty() && resolution.ambiguous.is_empty() {
        return Ok(());
    }
    let unresolved = resolution
        .unresolved
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let ambiguous = resolution
        .ambiguous
        .iter()
        .map(|value| value.input_id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(CliError::new(
        "RELINK_INCOMPLETE",
        format!("unresolved inputs: [{unresolved}]; ambiguous inputs: [{ambiguous}]"),
    ))
}
