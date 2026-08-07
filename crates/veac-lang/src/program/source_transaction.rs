use std::path::{Path, PathBuf};

use super::BuildInputManifestV1;
use crate::source_edit::SourceEditBatch;

mod executable;
mod model;
mod revision;
mod selection;

pub use model::{
    ExecutableSourceEditCandidate, ExecutableSourceEditPreview, SourceModuleChange,
    SourceTransactionError,
};

pub fn apply_executable_source_edit_path(
    entry: &Path,
    batch: &SourceEditBatch,
) -> Result<ExecutableSourceEditPreview, SourceTransactionError> {
    apply_executable_source_edit_path_with_inputs(entry, batch, &BuildInputManifestV1::empty())
}

pub fn apply_executable_source_edit_path_with_inputs(
    entry: &Path,
    batch: &SourceEditBatch,
    inputs: &BuildInputManifestV1,
) -> Result<ExecutableSourceEditPreview, SourceTransactionError> {
    apply_executable_source_edit_path_with_root_and_inputs(entry, batch, inputs)
        .map(|(_, preview)| preview)
}

pub fn apply_executable_source_edit_path_with_root(
    entry: &Path,
    batch: &SourceEditBatch,
) -> Result<(PathBuf, ExecutableSourceEditPreview), SourceTransactionError> {
    apply_executable_source_edit_path_with_root_and_inputs(
        entry,
        batch,
        &BuildInputManifestV1::empty(),
    )
}

pub fn apply_executable_source_edit_path_with_root_and_inputs(
    entry: &Path,
    batch: &SourceEditBatch,
    inputs: &BuildInputManifestV1,
) -> Result<(PathBuf, ExecutableSourceEditPreview), SourceTransactionError> {
    let (root, candidate) = prepare_executable_source_edit_path_with_root(entry, batch)?;
    candidate.execute(inputs).map(|preview| (root, preview))
}

pub fn prepare_executable_source_edit_path_with_root(
    entry: &Path,
    batch: &SourceEditBatch,
) -> Result<(PathBuf, ExecutableSourceEditCandidate), SourceTransactionError> {
    let (root, prepared) =
        super::prepare_path_with_root(entry).map_err(SourceTransactionError::Program)?;
    let (fallback, _) = super::FileSystemLoader::for_entry(entry)
        .map_err(|message| SourceTransactionError::Program(load_error(entry, message)))?;
    executable::prepare(&prepared, batch, &fallback).map(|candidate| (root, candidate))
}

fn load_error(entry: &Path, message: String) -> super::Diagnostics {
    super::Diagnostics::one(super::Diagnostic::new(
        "PROGRAM_ENTRY_LOAD",
        entry.display().to_string(),
        message,
        crate::authoring::Span::default(),
    ))
}
