use crate::source_edit::SourceEditBatch;

use super::super::loader::{LoadedSource, SourceLoader};
use super::super::{CompilerDatabase, ExecutableBuild, SourceIndex};
use super::loader::{CandidateLoader, ReplayLoader};
use super::model::{
    ExecutableSourceEditCandidate, ExecutableSourceEditPreview, SourceTransactionError,
};
use super::selection;

pub(super) fn reprepare(
    preview: &ExecutableSourceEditPreview,
    current_root: LoadedSource,
    fallback: &dyn SourceLoader,
) -> Result<ExecutableBuild, SourceTransactionError> {
    let overlay = preview
        .changes()
        .iter()
        .map(|change| (change.module().to_owned(), change.source().to_owned()))
        .collect();
    let loader = ReplayLoader::new(overlay, fallback);
    let root = preview.built.root_module();
    if current_root.id != root {
        return Err(SourceTransactionError::RootChanged {
            expected: root.to_owned(),
            actual: current_root.id,
        });
    }
    let source = preview
        .changes()
        .iter()
        .find(|change| change.module() == root)
        .map(|change| change.source().to_owned())
        .unwrap_or(current_root.source);
    super::super::prepare_with_loader(
        LoadedSource {
            id: root.to_owned(),
            source,
        },
        &loader,
    )
    .map_err(SourceTransactionError::Program)
}

pub(super) fn prepare_with_database(
    prepared: &ExecutableBuild,
    batch: &SourceEditBatch,
    fallback: &dyn SourceLoader,
    database: &CompilerDatabase,
) -> Result<ExecutableSourceEditCandidate, SourceTransactionError> {
    let complete =
        SourceIndex::build_snapshot(prepared.sources()).map_err(SourceTransactionError::Program)?;
    let revision = prepared
        .source_revision()
        .map_err(SourceTransactionError::Program)?;
    let selected = selection::apply(prepared.source_graph(), &revision, &complete, batch)?;
    let source = selected
        .overlay
        .get(prepared.root_module())
        .cloned()
        .unwrap_or_else(|| prepared.source_graph().root().source);
    let loader = CandidateLoader::new(selected.overlay, prepared.source_graph(), fallback);
    let candidate = super::super::executable::prepare_with_loader_and_database(
        LoadedSource {
            id: prepared.root_module().to_owned(),
            source,
        },
        &loader,
        database,
    )
    .map_err(SourceTransactionError::Program)?;
    Ok(ExecutableSourceEditCandidate::new(
        selected.changes,
        candidate,
        selected.previous_revision,
        selected.previous_modules,
        selected.previous_sources,
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
        let revision = built
            .source_revision()
            .map_err(SourceTransactionError::Program)?;
        let candidate_sources = built.source_graph().project_sources();
        Ok(ExecutableSourceEditPreview::new(
            self.changes,
            self.previous_revision,
            revision,
            built,
            self.previous_modules,
            self.previous_sources,
            candidate_sources,
        ))
    }
}
