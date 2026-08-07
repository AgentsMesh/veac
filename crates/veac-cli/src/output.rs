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
        if candidate.starts_with(&store_root) || store_root.starts_with(candidate) {
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

pub(crate) fn guarded_write_portable<'a>(
    path: &Path,
    protected: impl Iterator<Item = &'a Path>,
) -> CliResult<PathBuf> {
    let candidate = path::destination(path)?;
    let protected = protected.map(path::normalize_input).collect::<Vec<_>>();
    if aliases::conflicts_portable(&candidate, &protected)? {
        return Err(CliError::new(
            "OUTPUT_OVERWRITES_INPUT",
            format!("output {} aliases a project input", candidate.display()),
        ));
    }
    Ok(candidate)
}

pub(crate) fn guarded_package_directory<'a>(
    path: &Path,
    protected: impl Iterator<Item = &'a Path>,
) -> CliResult<PathBuf> {
    let candidate = path::normalize_input(&path::package_destination(path)?);
    let protected = protected.map(path::normalize_input).collect::<Vec<_>>();
    if protected.iter().any(|input| input.starts_with(&candidate)) {
        return Err(CliError::new(
            "OUTPUT_CONTAINS_INPUT",
            format!(
                "package output {} contains a project input",
                candidate.display()
            ),
        ));
    }
    Ok(candidate)
}

pub(crate) fn same_existing_or_equal(left: &Path, right: &Path) -> CliResult<bool> {
    let left = path::normalize_input(left);
    let right = path::normalize_input(right);
    Ok(left == right || aliases::same_existing(&left, &right)?)
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
                .map(|value| destination(value, &directory.join(target_name(value))))
                .collect()
        }
        None => deliverables
            .iter()
            .map(|value| destination(value, &project_directory.join(target_name(value))))
            .collect(),
    }
}

fn target_name(value: &veac_ir::Deliverable) -> &str {
    match &value.target {
        veac_ir::DeliverableTarget::File { name } => name,
        veac_ir::DeliverableTarget::ImageSequence { pattern } => pattern,
        veac_ir::DeliverableTarget::Package { name } => name,
    }
}

fn destination(value: &veac_ir::Deliverable, candidate: &Path) -> CliResult<PathBuf> {
    if matches!(value.target, veac_ir::DeliverableTarget::Package { .. }) {
        path::package_destination(candidate)
    } else {
        path::destination(candidate)
    }
}
