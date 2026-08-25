use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};
use veac_lang::program::ExecutableSourceEditPreview as EditPreview;

pub(super) fn revalidate_graph(
    root: &Path,
    package_roots: &[PathBuf],
    preview: &EditPreview,
) -> CliResult {
    let lock = crate::fs::SourceGraphReadLock::acquire(root)?;
    lock.revalidate(root)?;
    revalidate_graph_unlocked(root, package_roots, preview)?;
    lock.revalidate(root)
}

pub(super) fn commit_in_place(
    root: &Path,
    package_roots: &[PathBuf],
    preview: &EditPreview,
) -> CliResult {
    let source_lock = crate::fs::SourceGraphLock::acquire(root)?;
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
    source_lock.commit_modules_with(
        root,
        &changes,
        &guards,
        || revalidate_graph_unlocked(root, package_roots, preview),
        || revalidate_published(root, package_roots, preview),
    )
}

fn revalidate_graph_unlocked(
    root: &Path,
    package_roots: &[PathBuf],
    preview: &EditPreview,
) -> CliResult {
    let entry_path = root.join(preview.built.root_module());
    let snapshot = crate::frontend::source_loader_unlocked(&entry_path, package_roots)?;
    let actual = veac_lang::program::prepare_with_loader(snapshot.entry.clone(), &snapshot.loader)
        .map_err(|errors| crate::diagnostic::program(&entry_path, errors))?
        .source_revision()
        .map_err(|errors| crate::diagnostic::program(&entry_path, errors))?;
    if actual != preview.previous_revision {
        return Err(CliError::new(
            "SOURCE_CHANGED",
            format!(
                "source graph module {} changed after the edit was validated",
                root.join(preview.built.root_module()).display()
            ),
        ));
    }
    crate::fs::ensure_source_modules_unchanged(
        root,
        preview.built.root_module(),
        &guarded_sources(preview),
    )?;
    let candidate = map_source_changed(
        veac_lang::program::reprepare_executable_source_edit_preview(
            preview,
            snapshot.entry.clone(),
            &snapshot.loader,
        ),
        root,
        preview.built.root_module(),
    )?;
    require_candidate(root, preview, &candidate)?;
    snapshot.revalidate_packages()
}

fn revalidate_published(
    root: &Path,
    package_roots: &[PathBuf],
    preview: &EditPreview,
) -> CliResult {
    let entry_path = root.join(preview.built.root_module());
    let snapshot = crate::frontend::source_loader_unlocked(&entry_path, package_roots)?;
    let candidate = map_source_changed(
        veac_lang::program::prepare_with_loader(snapshot.entry.clone(), &snapshot.loader),
        root,
        preview.built.root_module(),
    )?;
    require_candidate(root, preview, &candidate)?;
    snapshot.revalidate_packages()
}

fn require_candidate(
    root: &Path,
    preview: &EditPreview,
    candidate: &veac_lang::program::ExecutableBuild,
) -> CliResult {
    let revision = map_source_changed(
        candidate.source_revision(),
        root,
        preview.built.root_module(),
    )?;
    if revision == preview.new_revision && candidate.source_graph() == preview.built.source_graph()
    {
        Ok(())
    } else {
        Err(source_changed(root, preview.built.root_module()))
    }
}

fn guarded_sources(preview: &EditPreview) -> Vec<(&str, &str)> {
    let mut guarded = std::collections::BTreeMap::new();
    for sources in [preview.previous_sources(), preview.candidate_sources()] {
        guarded.extend(
            sources
                .iter()
                .filter(|(module, _)| !is_changed(preview, module))
                .map(|(module, source)| (module.as_str(), source.as_str())),
        );
    }
    guarded.into_iter().collect()
}

fn is_changed(preview: &EditPreview, module: &str) -> bool {
    preview
        .changes()
        .iter()
        .any(|change| change.module() == module)
}

fn source_changed(root: &Path, module: &str) -> CliError {
    CliError::new(
        "SOURCE_CHANGED",
        format!(
            "candidate source graph module {} changed after validation",
            root.join(module).display()
        ),
    )
}

fn map_source_changed<T, E>(result: Result<T, E>, root: &Path, module: &str) -> CliResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(source_changed(root, module)),
    }
}

#[cfg(test)]
#[path = "revalidation/tests.rs"]
mod tests;
