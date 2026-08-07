use std::path::{Path, PathBuf};

use super::diagnostic::{Diagnostic, Diagnostics};
use super::expression::ExecutionBudget;
use super::loader::{FileSystemLoader, LoadedSource, MemoryLoader, SourceLoader};
use super::resolve;

mod entry;
mod identity;
mod lower;
mod model;
mod temporal;

pub(crate) use entry::validate as validate_entry;
use model::ExecutableRegistries;
pub use model::{BuiltProgram, ExecutableBuild};
use temporal::{compile_attached, AttachedTemporalLeaf};
pub(crate) use temporal::{
    ClipTemporalProperty, ExecutableTemporalLeaf, ExecutableTemporalSink, MaskTemporalProperty,
    TextTemporalProperty,
};

pub fn prepare_path(path: &Path) -> Result<ExecutableBuild, Diagnostics> {
    prepare_path_with_root(path).map(|(_, build)| build)
}

pub fn prepare_path_with_root(path: &Path) -> Result<(PathBuf, ExecutableBuild), Diagnostics> {
    let (loader, entry) = FileSystemLoader::for_entry(path).map_err(|message| {
        Diagnostics::one(Diagnostic::new(
            "PROGRAM_ENTRY_LOAD",
            path.display().to_string(),
            message,
            crate::authoring::Span::default(),
        ))
    })?;
    let root = loader.root().to_owned();
    prepare_with_loader(entry, &loader).map(|build| (root, build))
}

pub fn prepare_source(source: &str) -> Result<ExecutableBuild, Diagnostics> {
    super::limits::check_source_size("main.veac", source, crate::authoring::Span::default())
        .map_err(Diagnostics::one)?;
    prepare_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: source.to_owned(),
        },
        &MemoryLoader::default(),
    )
}

pub fn prepare_with_loader(
    entry: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<ExecutableBuild, Diagnostics> {
    let root_module = entry.id.clone();
    let resolution = resolve::executable_entry(entry, loader, &ExecutionBudget::default())
        .map_err(Diagnostics)?;
    let main = resolution
        .scope
        .functions
        .lookup("main")
        .expect("validated executable main is resolved")
        .clone();
    let temporal_leaves =
        temporal::compile_authored(&resolution.entry, &resolution.scope.expression_context())?;
    Ok(ExecutableBuild::new(
        root_module,
        resolution.sources,
        main,
        ExecutableRegistries::new(
            resolution.scope.functions,
            resolution.scope.methods,
            resolution.scope.types,
            resolution.scope.build_inputs,
        ),
        temporal_leaves,
    ))
}

pub fn build_path(path: &Path) -> Result<BuiltProgram, Diagnostics> {
    prepare_path(path)?.execute()
}

pub fn build_path_with_inputs(
    path: &Path,
    inputs: &super::BuildInputManifestV1,
) -> Result<BuiltProgram, Diagnostics> {
    prepare_path(path)?.execute_with_inputs(inputs)
}

pub fn build_path_with_root_and_inputs(
    path: &Path,
    inputs: &super::BuildInputManifestV1,
) -> Result<(PathBuf, BuiltProgram), Diagnostics> {
    let (root, prepared) = prepare_path_with_root(path)?;
    prepared
        .execute_with_inputs(inputs)
        .map(|built| (root, built))
}

pub fn build_path_with_root(path: &Path) -> Result<(PathBuf, BuiltProgram), Diagnostics> {
    let (root, prepared) = prepare_path_with_root(path)?;
    prepared.execute().map(|built| (root, built))
}

pub fn build_source(source: &str) -> Result<BuiltProgram, Diagnostics> {
    prepare_source(source)?.execute()
}

pub fn build_source_with_inputs(
    source: &str,
    inputs: &super::BuildInputManifestV1,
) -> Result<BuiltProgram, Diagnostics> {
    prepare_source(source)?.execute_with_inputs(inputs)
}

pub fn build_with_loader(
    entry: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<BuiltProgram, Diagnostics> {
    prepare_with_loader(entry, loader)?.execute()
}

#[cfg(test)]
mod tests;
