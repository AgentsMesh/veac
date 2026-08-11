mod decode;
mod error;
mod loader;
mod provenance;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use error::{ProjectAuthoringError, ProjectDecodeError};
use loader::ProjectLoader;
use veac_lang::program::{
    prepare_host_with_loader, EntryContract, EntryValueType, FileSystemLoader, LoadedSource,
    SourceIndex, SourceLoader,
};
use veac_lang::source_edit::SourceRevision;

use crate::ProjectManifestV1;

pub const PROJECT_MODULE_ID: &str = "veac/project.veac";
pub const PROJECT_MODULE_SOURCE: &str = include_str!("../project.veac");

#[derive(Debug, Clone)]
pub struct AuthoredProjectManifest {
    pub manifest: ProjectManifestV1,
    pub root_module: String,
    pub sources: BTreeMap<String, String>,
    pub source_revision: SourceRevision,
    pub source_index: SourceIndex,
    pub manifest_digest: String,
}

pub fn project_entry_contract() -> EntryContract {
    EntryContract::new(
        "workspace",
        EntryValueType::nominal_from(PROJECT_MODULE_ID, "ProjectManifest"),
    )
    .with_prelude(PROJECT_MODULE_ID)
}

pub fn build_project_source(
    source: &str,
) -> Result<AuthoredProjectManifest, ProjectAuthoringError> {
    build_project_with_loader(
        LoadedSource {
            id: "project.veac".to_owned(),
            source: source.to_owned(),
        },
        &loader::RejectLoader,
    )
}

pub fn build_project_path(
    path: &Path,
) -> Result<(PathBuf, AuthoredProjectManifest), ProjectAuthoringError> {
    let (loader, entry) = FileSystemLoader::for_entry(path).map_err(ProjectAuthoringError::Load)?;
    let root = loader.root().to_owned();
    build_project_with_loader(entry, &loader).map(|project| (root, project))
}

pub fn build_project_root_path(
    root: &Path,
    entry: &Path,
) -> Result<AuthoredProjectManifest, ProjectAuthoringError> {
    let (loader, source) =
        FileSystemLoader::for_root_entry(root, entry).map_err(ProjectAuthoringError::Load)?;
    build_project_with_loader(source, &loader)
}

pub fn build_project_with_loader(
    entry: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<AuthoredProjectManifest, ProjectAuthoringError> {
    let overlay = ProjectLoader::new(loader);
    let prepared = prepare_host_with_loader(entry, &overlay, &project_entry_contract())?;
    let root_module = prepared.root_module().to_owned();
    let sources = provenance::authored_sources(prepared.sources());
    let evaluated = prepared.execute(&[])?;
    let manifest = decode::manifest(evaluated.value(), evaluated.type_registry())?;
    crate::validate_manifest(&manifest)?;
    provenance::finish(manifest, root_module, sources)
}
