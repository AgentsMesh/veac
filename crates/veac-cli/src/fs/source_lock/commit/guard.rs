use std::path::{Path, PathBuf};

use super::super::path::{self, Parent, Target};
use super::super::SourceGraphLock;
use super::{require_source, SourceModuleGuard};
use crate::error::CliResult;

pub(super) struct Guarded {
    module: String,
    label: PathBuf,
    parent: Parent,
    original: Target,
}

pub(super) fn prepare(
    lock: &SourceGraphLock,
    root: &Path,
    values: &[SourceModuleGuard<'_>],
) -> CliResult<Vec<Guarded>> {
    values
        .iter()
        .map(|value| {
            let label = root.join(value.module);
            let parent = path::resolve(&lock.directory, value.module, &label)?;
            let original = path::read_target(&parent, &label, value.expected.len())?;
            require_source(&label, &original.bytes, value.expected.as_bytes())?;
            Ok(Guarded {
                module: value.module.to_owned(),
                label,
                parent,
                original,
            })
        })
        .collect()
}

pub(super) fn verify(
    lock: &SourceGraphLock,
    prepared: &[Guarded],
    values: &[SourceModuleGuard<'_>],
) -> CliResult {
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
                "guarded module identity or mode changed during batch commit",
            ));
        }
        require_source(&item.label, &current.bytes, value.expected.as_bytes())?;
    }
    Ok(())
}
