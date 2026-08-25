use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};
use veac_lang::program::ExecutableSourceEditPreview as EditPreview;

mod revalidation;

pub(super) struct Context<'a> {
    pub(super) root: &'a Path,
    pub(super) entry: &'a Path,
    pub(super) batch: &'a Path,
    pub(super) inputs: Option<&'a Path>,
    pub(super) package_roots: &'a [PathBuf],
}

pub(super) fn publish(
    context: Context<'_>,
    preview: &EditPreview,
    output: Option<&Path>,
    dry_run: bool,
) -> CliResult<Vec<String>> {
    if dry_run {
        revalidation::revalidate_graph(context.root, context.package_roots, preview)?;
        return Ok(Vec::new());
    }
    let Some(output) = output else {
        revalidation::commit_in_place(context.root, context.package_roots, preview)?;
        return Ok(preview
            .changes()
            .iter()
            .map(|change| context.root.join(change.module()).display().to_string())
            .collect());
    };
    write_independent(context, preview, output)
}

fn write_independent(
    context: Context<'_>,
    preview: &EditPreview,
    output: &Path,
) -> CliResult<Vec<String>> {
    let [change] = preview.changes() else {
        return Err(CliError::new(
            "SOURCE_EDIT_OUTPUT_REQUIRES_SINGLE_MODULE",
            "--output accepts one changed module; multi-module batches must commit in place",
        ));
    };
    let target = module_path(context.entry, change.module())?;
    let protected = protected_paths(
        context.root,
        context.entry,
        context.batch,
        context.inputs,
        preview,
    )?;
    let destination = destination(output, &target, protected)?;
    protect_package_sources(&destination, context.package_roots)?;
    let aliases_target = crate::output::same_existing_or_equal(&destination, &target)?;
    if aliases_target && destination != target {
        return Err(CliError::new(
            "OUTPUT_OVERWRITES_INPUT",
            format!("output {} aliases the edited source", destination.display()),
        ));
    }
    if destination == target {
        revalidation::commit_in_place(context.root, context.package_roots, preview)?;
    } else {
        revalidation::revalidate_graph(context.root, context.package_roots, preview)?;
        crate::fs::atomic_write(&destination, change.source())?;
    }
    Ok(vec![destination.display().to_string()])
}

fn protected_paths(
    root: &Path,
    entry: &Path,
    batch: &Path,
    inputs: Option<&Path>,
    preview: &EditPreview,
) -> CliResult<Vec<PathBuf>> {
    let mut protected = Vec::new();
    for sources in [preview.previous_sources(), preview.candidate_sources()] {
        protected.extend(sources.keys().map(|module| root.join(module)));
    }
    protected.sort();
    protected.dedup();
    protected.push(batch.to_path_buf());
    if let Some(inputs) = inputs {
        protected.push(crate::fs::canonical_file(inputs, "Build input manifest")?);
    }
    protected.extend(crate::canonical::local_material_paths(
        preview.built.envelope(),
        entry,
    ));
    protected.push(root.join(crate::fs::SOURCE_LOCK_NAME));
    Ok(protected)
}

fn module_path(entry: &Path, module: &str) -> CliResult<PathBuf> {
    veac_lang::source_edit::validate_module_path(module)
        .map_err(|error| CliError::new("SOURCE_EDIT_MODULE", error.to_string()))?;
    Ok(entry
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(module))
}

fn destination(output: &Path, target: &Path, protected: Vec<PathBuf>) -> CliResult<PathBuf> {
    let protected = protected
        .into_iter()
        .filter(|path| path != target)
        .collect::<Vec<_>>();
    crate::output::guarded_write_portable(output, protected.iter().map(PathBuf::as_path))
}

fn protect_package_sources(output: &Path, package_roots: &[PathBuf]) -> CliResult {
    for root in package_roots {
        let root = crate::fs::canonical_directory(root, "VEAC package root")?;
        if output.starts_with(&root) {
            return Err(CliError::new(
                "OUTPUT_OVERWRITES_INPUT",
                format!(
                    "output {} is inside a read-only package root",
                    output.display()
                ),
            ));
        }
    }
    Ok(())
}
