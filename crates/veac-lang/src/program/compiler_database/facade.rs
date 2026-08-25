use std::path::{Path, PathBuf};

use super::CompilerDatabase;
use crate::program::loader::{LoadedSource, SourceLoader};
use crate::program::{
    BuiltProgram, Diagnostics, ExecutableBuild, ExecutableSourceEditCandidate,
    SourceTransactionError,
};
use crate::source_edit::SourceEditBatch;

impl CompilerDatabase {
    pub fn prepare_path(&self, path: &Path) -> Result<ExecutableBuild, Diagnostics> {
        crate::program::executable::prepare_path_with_database(path, self)
    }

    pub fn prepare_path_with_root(
        &self,
        path: &Path,
    ) -> Result<(PathBuf, ExecutableBuild), Diagnostics> {
        crate::program::executable::prepare_path_with_root_and_database(path, self)
    }

    pub fn prepare_source(&self, source: &str) -> Result<ExecutableBuild, Diagnostics> {
        crate::program::executable::prepare_source_with_database(source, self)
    }

    pub fn prepare_with_loader(
        &self,
        entry: LoadedSource,
        loader: &dyn SourceLoader,
    ) -> Result<ExecutableBuild, Diagnostics> {
        crate::program::executable::prepare_with_loader_and_database(entry, loader, self)
    }

    pub fn build_path(&self, path: &Path) -> Result<BuiltProgram, Diagnostics> {
        self.prepare_path(path)?.execute()
    }

    pub fn build_path_with_inputs(
        &self,
        path: &Path,
        inputs: &crate::program::BuildInputManifestV1,
    ) -> Result<BuiltProgram, Diagnostics> {
        self.prepare_path(path)?.execute_with_inputs(inputs)
    }

    pub fn build_path_with_root(
        &self,
        path: &Path,
    ) -> Result<(PathBuf, BuiltProgram), Diagnostics> {
        let (root, prepared) = self.prepare_path_with_root(path)?;
        prepared.execute().map(|built| (root, built))
    }

    pub fn build_source(&self, source: &str) -> Result<BuiltProgram, Diagnostics> {
        self.prepare_source(source)?.execute()
    }

    pub fn build_source_with_inputs(
        &self,
        source: &str,
        inputs: &crate::program::BuildInputManifestV1,
    ) -> Result<BuiltProgram, Diagnostics> {
        self.prepare_source(source)?.execute_with_inputs(inputs)
    }

    pub fn build_with_loader(
        &self,
        entry: LoadedSource,
        loader: &dyn SourceLoader,
    ) -> Result<BuiltProgram, Diagnostics> {
        self.prepare_with_loader(entry, loader)?.execute()
    }

    pub fn prepare_source_edit_path(
        &self,
        path: &Path,
        batch: &SourceEditBatch,
    ) -> Result<(PathBuf, ExecutableSourceEditCandidate), SourceTransactionError> {
        crate::program::source_transaction::prepare_executable_source_edit_path_with_root_and_database(
            path, batch, self,
        )
    }
}
