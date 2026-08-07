use crate::source_edit::SourceEditBatch;

use super::super::loader::{LoadedSource, MemoryLoader, SourceLoader};
use super::super::ExecutableBuild;
use super::model::{
    ExecutableSourceEditCandidate, ExecutableSourceEditPreview, SourceTransactionError,
};
use super::selection;

pub(super) fn prepare(
    prepared: &ExecutableBuild,
    batch: &SourceEditBatch,
    fallback: &dyn SourceLoader,
) -> Result<ExecutableSourceEditCandidate, SourceTransactionError> {
    let index = prepared
        .source_index()
        .map_err(SourceTransactionError::Program)?;
    let selected = selection::apply(prepared.sources(), &index, batch)?;
    let source = selected
        .sources
        .get(prepared.root_module())
        .cloned()
        .ok_or(SourceTransactionError::TargetNotFound { operation: 0 })?;
    let loader = CandidateLoader {
        overlay: MemoryLoader::new(selected.sources),
        fallback,
    };
    let candidate = super::super::prepare_with_loader(
        LoadedSource {
            id: prepared.root_module().to_owned(),
            source,
        },
        &loader,
    )
    .map_err(SourceTransactionError::Program)?;
    Ok(ExecutableSourceEditCandidate::new(
        selected.changes,
        candidate,
        selected.previous_revision,
        selected.previous_modules,
    ))
}

impl ExecutableSourceEditCandidate {
    pub fn execute(
        self,
        inputs: &super::super::BuildInputManifestV1,
    ) -> Result<ExecutableSourceEditPreview, SourceTransactionError> {
        let built = self
            .prepared
            .execute_with_inputs(inputs)
            .map_err(SourceTransactionError::Program)?;
        if let Some(change) = self
            .changes
            .iter()
            .find(|change| !built.sources().contains_key(change.module()))
        {
            return Err(SourceTransactionError::ChangedModuleUnreachable {
                module: change.module().to_owned(),
            });
        }
        let revision = super::revision::calculate(built.sources())
            .map_err(SourceTransactionError::Contract)?;
        Ok(ExecutableSourceEditPreview::new(
            self.changes,
            self.previous_revision,
            revision,
            built,
            self.previous_modules,
        ))
    }
}

struct CandidateLoader<'a> {
    overlay: MemoryLoader,
    fallback: &'a dyn SourceLoader,
}

impl SourceLoader for CandidateLoader<'_> {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        self.overlay
            .load(importer, requested)
            .or_else(|_| self.fallback.load(importer, requested))
    }
}
