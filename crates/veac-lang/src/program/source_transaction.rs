use std::path::{Path, PathBuf};

use super::BuildInputManifestV1;
use crate::source_edit::SourceEditBatch;

mod executable;
mod loader;
mod model;
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
    prepare_executable_source_edit_path_with_root_and_database(
        entry,
        batch,
        &super::CompilerDatabase::default(),
    )
}

pub fn prepare_executable_source_edit_path_with_root_and_database(
    entry: &Path,
    batch: &SourceEditBatch,
    database: &super::CompilerDatabase,
) -> Result<(PathBuf, ExecutableSourceEditCandidate), SourceTransactionError> {
    let (root, prepared) = super::executable::prepare_path_with_root_and_database(entry, database)
        .map_err(SourceTransactionError::Program)?;
    let (fallback, _) = super::FileSystemLoader::for_entry(entry)
        .map_err(|message| SourceTransactionError::Program(load_error(entry, message)))?;
    executable::prepare_with_database(&prepared, batch, &fallback, database)
        .map(|candidate| (root, candidate))
}

pub fn prepare_executable_source_edit_with_loader(
    prepared: &super::ExecutableBuild,
    batch: &SourceEditBatch,
    loader: &dyn super::SourceLoader,
) -> Result<ExecutableSourceEditCandidate, SourceTransactionError> {
    executable::prepare_with_database(prepared, batch, loader, &super::CompilerDatabase::default())
}

pub fn reprepare_executable_source_edit_preview(
    preview: &ExecutableSourceEditPreview,
    current_root: super::LoadedSource,
    loader: &dyn super::SourceLoader,
) -> Result<super::ExecutableBuild, SourceTransactionError> {
    executable::reprepare(preview, current_root, loader)
}

fn load_error(entry: &Path, message: String) -> super::Diagnostics {
    super::Diagnostics::one(super::Diagnostic::new(
        "PROGRAM_ENTRY_LOAD",
        entry.display().to_string(),
        message,
        crate::authoring::Span::default(),
    ))
}
