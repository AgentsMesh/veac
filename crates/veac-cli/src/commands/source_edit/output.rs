use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};
use veac_lang::program::ExecutableSourceEditPreview as EditPreview;

pub(super) fn publish(
    root: &Path,
    entry: &Path,
    batch: &Path,
    inputs: Option<&Path>,
    preview: &EditPreview,
    output: Option<&Path>,
    dry_run: bool,
) -> CliResult<Vec<String>> {
    if dry_run {
        revalidate_graph(root, preview)?;
        return Ok(Vec::new());
    }
    let Some(output) = output else {
        commit_in_place(root, preview)?;
        return Ok(preview
            .changes()
            .iter()
            .map(|change| root.join(change.module()).display().to_string())
            .collect());
    };
    write_independent(root, entry, batch, inputs, preview, output)
}

fn write_independent(
    root: &Path,
    entry: &Path,
    batch: &Path,
    inputs: Option<&Path>,
    preview: &EditPreview,
    output: &Path,
) -> CliResult<Vec<String>> {
    let [change] = preview.changes() else {
        return Err(CliError::new(
            "SOURCE_EDIT_OUTPUT_REQUIRES_SINGLE_MODULE",
            "--output accepts one changed module; multi-module batches must commit in place",
        ));
    };
    let target = module_path(entry, change.module())?;
    let protected = protected_paths(root, entry, batch, inputs, preview)?;
    let destination = destination(output, &target, protected)?;
    let aliases_target = crate::output::same_existing_or_equal(&destination, &target)?;
    if aliases_target && destination != target {
        return Err(CliError::new(
            "OUTPUT_OVERWRITES_INPUT",
            format!("output {} aliases the edited source", destination.display()),
        ));
    }
    if destination == target {
        commit_in_place(root, preview)?;
    } else {
        revalidate_graph(root, preview)?;
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
    let mut protected = preview
        .built
        .sources()
        .keys()
        .map(|module| root.join(module))
        .collect::<Vec<_>>();
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

fn revalidate_graph(root: &Path, preview: &EditPreview) -> CliResult {
    crate::fs::ensure_source_graph_unchanged(
        root,
        preview.built.root_module(),
        preview.previous_modules(),
        &preview.previous_revision,
    )?;
    crate::fs::ensure_source_modules_unchanged(
        root,
        preview.built.root_module(),
        &guarded_sources(preview),
    )
}

fn commit_in_place(root: &Path, preview: &EditPreview) -> CliResult {
    let source_lock = crate::fs::SourceGraphLock::acquire(root)?;
    source_lock.revalidate(root)?;
    revalidate_graph(root, preview)?;
    source_lock.revalidate(root)?;
    let changes = preview
        .changes()
        .iter()
        .map(|change| crate::fs::SourceModuleReplacement {
            module: change.module(),
            expected: change.previous_source(),
            replacement: change.source(),
        })
        .collect::<Vec<_>>();
    let guards = guarded_sources(preview)
        .into_iter()
        .map(|(module, expected)| crate::fs::SourceModuleGuard { module, expected })
        .collect::<Vec<_>>();
    source_lock.commit_modules(root, &changes, &guards)
}

fn guarded_sources(preview: &EditPreview) -> Vec<(&str, &str)> {
    preview
        .built
        .sources()
        .iter()
        .filter(|(module, _)| {
            !preview
                .changes()
                .iter()
                .any(|change| change.module() == module.as_str())
        })
        .map(|(module, source)| (module.as_str(), source.as_str()))
        .collect()
}

fn destination(output: &Path, target: &Path, protected: Vec<PathBuf>) -> CliResult<PathBuf> {
    let protected = protected
        .into_iter()
        .filter(|path| path != target)
        .collect::<Vec<_>>();
    crate::output::guarded_write_portable(output, protected.iter().map(PathBuf::as_path))
}
