mod decode;
mod error;
mod loader;
mod provenance;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub use error::{EvidenceAuthoringError, EvidenceDecodeError};
use loader::EvidenceLoader;
use veac_lang::program::{
    prepare_host_with_loader, EntryContract, EntryValueType, FileSystemLoader, LoadedSource,
    SourceIndex, SourceLoader,
};
use veac_lang::source_edit::SourceRevision;

use crate::EvidenceSuiteV1;

pub const EVIDENCE_MODULE_ID: &str = "veac/evidence.veac";
pub const EVIDENCE_MODULE_SOURCE: &str = include_str!("../evidence.veac");

#[derive(Debug, Clone)]
pub struct AuthoredEvidenceSuite {
    pub suite: EvidenceSuiteV1,
    pub root_module: String,
    pub sources: BTreeMap<String, String>,
    pub source_revision: SourceRevision,
    pub source_index: SourceIndex,
    pub suite_sha256: String,
}

pub fn evidence_entry_contract() -> EntryContract {
    EntryContract::new(
        "evidence",
        EntryValueType::nominal_from(EVIDENCE_MODULE_ID, "EvidenceSuite"),
    )
    .with_prelude(EVIDENCE_MODULE_ID)
}

pub fn build_evidence_source(
    source: &str,
) -> Result<AuthoredEvidenceSuite, EvidenceAuthoringError> {
    build_evidence_with_loader(
        LoadedSource {
            id: "evidence.veac".to_owned(),
            source: source.to_owned(),
        },
        &loader::RejectLoader,
    )
}

pub fn build_evidence_path(
    path: &Path,
) -> Result<(PathBuf, AuthoredEvidenceSuite), EvidenceAuthoringError> {
    let (loader, entry) =
        FileSystemLoader::for_entry(path).map_err(EvidenceAuthoringError::Load)?;
    let root = loader.root().to_owned();
    build_evidence_with_loader(entry, &loader).map(|suite| (root, suite))
}

pub fn build_evidence_root_path(
    root: &Path,
    entry: &Path,
) -> Result<AuthoredEvidenceSuite, EvidenceAuthoringError> {
    let (loader, source) =
        FileSystemLoader::for_root_entry(root, entry).map_err(EvidenceAuthoringError::Load)?;
    build_evidence_with_loader(source, &loader)
}

pub fn build_evidence_with_loader(
    entry: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<AuthoredEvidenceSuite, EvidenceAuthoringError> {
    let overlay = EvidenceLoader::new(loader);
    let prepared = prepare_host_with_loader(entry, &overlay, &evidence_entry_contract())?;
    let root_module = prepared.root_module().to_owned();
    let sources = provenance::authored_sources(prepared.sources());
    let evaluated = prepared.execute(&[])?;
    let suite = decode::suite(evaluated.value(), evaluated.type_registry())?;
    let suite = crate::validate(suite)?.into_suite();
    provenance::finish(suite, root_module, sources)
}
