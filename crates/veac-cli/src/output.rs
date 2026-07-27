use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};
use crate::planning::PreparedPlan;

mod aliases;
mod contract;
mod path;

pub(crate) fn bind_render_outputs(
    prepared: &mut PreparedPlan,
    destination: Option<&Path>,
) -> CliResult<Vec<PathBuf>> {
    let deliverables = &prepared.plan.output.deliverables;
    let candidates = candidates(
        prepared.project_file.parent().unwrap_or(Path::new(".")),
        deliverables,
        destination,
    )?;
    let protected = std::iter::once(prepared.project_file.as_path())
        .chain(prepared.binding_file.as_deref())
        .chain(prepared.material_paths.values().map(PathBuf::as_path))
        .map(path::normalize_input)
        .collect::<Vec<_>>();
    let store_root = path::normalize_input(
        &prepared
            .project_file
            .parent()
            .unwrap_or(Path::new("."))
            .join(".veac-artifacts"),
    );
    for (deliverable, candidate) in deliverables.iter().zip(&candidates) {
        contract::validate(deliverable, candidate, &protected)?;
        if candidate.starts_with(&store_root) {
            return Err(CliError::new(
                "OUTPUT_RESERVED_DIRECTORY",
                format!(
                    "output {} is inside the artifact store",
                    candidate.display()
                ),
            ));
        }
    }
    for (deliverable, path) in deliverables.iter().zip(&candidates) {
        prepared
            .bindings
            .bind_output(deliverable.id.clone(), path.clone())
            .map_err(|error| CliError::new("OUTPUT_BINDING_FAILED", error.to_string()))?;
    }
    Ok(candidates)
}

pub(crate) fn guarded_write_many<'a>(
    path: &Path,
    protected: impl Iterator<Item = &'a Path>,
) -> CliResult<PathBuf> {
    let candidate = path::destination(path)?;
    let protected: Vec<_> = protected.map(path::normalize_input).collect();
    if aliases::conflicts(&candidate, &protected)? {
        return Err(CliError::new(
            "OUTPUT_OVERWRITES_INPUT",
            format!("output {} aliases a project input", candidate.display()),
        ));
    }
    Ok(candidate)
}

fn candidates(
    project_directory: &Path,
    deliverables: &[veac_ir::Deliverable],
    requested: Option<&Path>,
) -> CliResult<Vec<PathBuf>> {
    if deliverables.is_empty() {
        return Err(CliError::new(
            "OUTPUT_DELIVERABLE_MISSING",
            "resolved plan has no deliverables",
        ));
    }
    match requested {
        Some(directory) => {
            let directory = path::output_directory(directory)?;
            deliverables
                .iter()
                .map(|value| path::destination(&directory.join(&value.file_name)))
                .collect()
        }
        None => deliverables
            .iter()
            .map(|value| path::destination(&project_directory.join(&value.file_name)))
            .collect(),
    }
}
