use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::super::path::{self, Parent, Target};
use super::super::stage::Staged;
use super::super::SourceGraphLock;
use super::guard;
use super::{SourceModuleGuard, SourceModuleReplacement};
use crate::error::{CliError, CliResult};

struct Prepared {
    module: String,
    label: PathBuf,
    parent: Parent,
    original: Target,
}

pub(super) fn commit(
    lock: &SourceGraphLock,
    root: &Path,
    values: &[SourceModuleReplacement<'_>],
    guards: &[SourceModuleGuard<'_>],
) -> CliResult {
    if values.is_empty() {
        return Err(CliError::new(
            "SOURCE_EDIT_REJECTED",
            "source edit commit has no changed modules",
        ));
    }
    require_unique(values, guards)?;
    lock.revalidate(root)?;
    let guarded = guard::prepare(lock, root, guards)?;
    let prepared = prepare(lock, root, values)?;
    let replacements = stage(&prepared, values, false)?;
    let rollbacks = stage(&prepared, values, true)?;
    final_check(lock, root, &prepared, values)?;
    guard::verify(lock, &guarded, guards)?;
    publish(
        lock,
        root,
        &prepared,
        replacements,
        rollbacks,
        &guarded,
        guards,
    )
}

fn prepare(
    lock: &SourceGraphLock,
    root: &Path,
    values: &[SourceModuleReplacement<'_>],
) -> CliResult<Vec<Prepared>> {
    values
        .iter()
        .map(|value| {
            let label = root.join(value.module);
            let parent = path::resolve(&lock.directory, value.module, &label)?;
            let original = path::read_target(&parent, &label, value.expected.len())?;
            super::require_source(&label, &original.bytes, value.expected.as_bytes())?;
            Ok(Prepared {
                module: value.module.to_owned(),
                label,
                parent,
                original,
            })
        })
        .collect()
}

fn stage<'a>(
    prepared: &'a [Prepared],
    values: &[SourceModuleReplacement<'_>],
    rollback: bool,
) -> CliResult<Vec<Staged<'a>>> {
    prepared
        .iter()
        .zip(values)
        .map(|(item, value)| {
            let bytes = if rollback {
                item.original.bytes.as_slice()
            } else {
                value.replacement.as_bytes()
            };
            Staged::create(
                &item.parent.descriptor,
                item.original.mode,
                bytes,
                &item.label,
            )
        })
        .collect()
}

fn final_check(
    lock: &SourceGraphLock,
    root: &Path,
    prepared: &[Prepared],
    values: &[SourceModuleReplacement<'_>],
) -> CliResult {
    lock.revalidate(root)?;
    for (item, value) in prepared.iter().zip(values) {
        path::require_parent(
            &lock.directory,
            &item.module,
            item.parent.identity,
            &item.label,
        )?;
        let current = path::read_target(&item.parent, &item.label, value.expected.len())?;
        if current.identity != item.original.identity || current.mode != item.original.mode {
            return Err(super::changed(
                &item.label,
                "module identity or mode changed before batch commit",
            ));
        }
        super::require_source(&item.label, &current.bytes, value.expected.as_bytes())?;
    }
    Ok(())
}

fn publish(
    lock: &SourceGraphLock,
    root: &Path,
    prepared: &[Prepared],
    replacements: Vec<Staged<'_>>,
    rollbacks: Vec<Staged<'_>>,
    guarded: &[guard::Guarded],
    guards: &[SourceModuleGuard<'_>],
) -> CliResult {
    let mut rollbacks = rollbacks.into_iter().map(Some).collect::<Vec<_>>();
    let mut published = 0usize;
    for (index, (staged, item)) in replacements.into_iter().zip(prepared).enumerate() {
        if let Err(error) = staged.publish(&item.parent, &item.label) {
            let uncertain = error.diagnostics()[0].code == "WRITE_COMMIT_UNCERTAIN";
            let count = published + usize::from(uncertain);
            return rollback(prepared, &mut rollbacks, count, error);
        }
        published = index + 1;
    }
    lock.revalidate(root).map_err(super::committed)?;
    for item in prepared {
        path::require_parent(
            &lock.directory,
            &item.module,
            item.parent.identity,
            &item.label,
        )
        .map_err(super::committed)?;
    }
    guard::verify(lock, guarded, guards).map_err(super::committed)?;
    Ok(())
}

fn rollback(
    prepared: &[Prepared],
    rollbacks: &mut [Option<Staged<'_>>],
    published: usize,
    original: CliError,
) -> CliResult {
    for index in (0..published).rev() {
        let staged = rollbacks[index].take().expect("rollback stage is present");
        if let Err(error) = staged.publish(&prepared[index].parent, &prepared[index].label) {
            return Err(CliError::new(
                "WRITE_COMMIT_UNCERTAIN",
                format!("{original}; batch rollback also failed: {error}"),
            ));
        }
    }
    Err(original)
}

fn require_unique(
    values: &[SourceModuleReplacement<'_>],
    guards: &[SourceModuleGuard<'_>],
) -> CliResult {
    let mut modules = BTreeSet::new();
    let unique_changes = values.iter().all(|value| modules.insert(value.module));
    if unique_changes && guards.iter().all(|value| modules.insert(value.module)) {
        Ok(())
    } else {
        Err(CliError::new(
            "SOURCE_EDIT_REJECTED",
            "source edit commit contains a duplicate module",
        ))
    }
}

#[cfg(test)]
#[path = "batch/tests.rs"]
mod tests;
