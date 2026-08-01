use std::time::Instant;

use veac_codegen::emitter::{BackendCommand, BackendFilterBinding, BackendInput};

use super::{numbered, snapshot_error, ResourceSnapshots};
use crate::executor::contract;
use crate::RuntimeError;

pub(super) fn command(
    snapshots: &ResourceSnapshots,
    value: &mut BackendCommand,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    value.inputs = value
        .inputs
        .iter()
        .map(|input| {
            snapshots
                .file(&input.path)
                .map(|path| BackendInput { path: path.clone() })
        })
        .collect::<Result<_, _>>()?;
    for preparation in &mut value.preparations {
        command(snapshots, &mut preparation.command, deadline)?;
    }
    if let Some(contract) = &value.filter_contract {
        let graph = contract
            .render_bound(&snapshots.files, &snapshots.directories)
            .map_err(|error| RuntimeError::new(format!("cannot bind filter snapshot: {error}")))?;
        contract::filter::validate_size(&graph)?;
        value.filter_graph = Some(graph);
    }
    super::super::deadline::ensure(deadline)
}

pub(super) fn directories(
    snapshots: &mut ResourceSnapshots,
    command: &BackendCommand,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    if let Some(contract) = &command.filter_contract {
        for binding in contract.bindings() {
            super::super::deadline::ensure_setup(deadline)?;
            let BackendFilterBinding::Directory { files, .. } = binding else {
                continue;
            };
            if snapshots.directories.contains_key(files) {
                continue;
            }
            let path = snapshots
                ._directory
                .path()
                .join(format!("fonts-{:04}", snapshots.directories.len()));
            std::fs::create_dir(&path).map_err(snapshot_error)?;
            for (index, original) in files.iter().enumerate() {
                super::super::deadline::ensure_setup(deadline)?;
                let source = snapshots.file(original)?;
                let destination = numbered(&path, "font", index, original);
                std::fs::hard_link(source, destination).map_err(snapshot_error)?;
            }
            snapshots.directories.insert(files.clone(), path);
        }
    }
    for preparation in &command.preparations {
        directories(snapshots, &preparation.command, deadline)?;
    }
    Ok(())
}
