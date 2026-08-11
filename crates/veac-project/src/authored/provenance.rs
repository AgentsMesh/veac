use std::collections::BTreeMap;

use veac_lang::program::SourceIndex;

use super::{AuthoredProjectManifest, ProjectAuthoringError, PROJECT_MODULE_ID};
use crate::ProjectManifestV1;

pub(super) fn authored_sources(resolved: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    resolved
        .iter()
        .filter(|(path, _)| path.as_str() != PROJECT_MODULE_ID)
        .map(|(path, source)| (path.clone(), source.clone()))
        .collect()
}

pub(super) fn finish(
    manifest: ProjectManifestV1,
    root_module: String,
    sources: BTreeMap<String, String>,
) -> Result<AuthoredProjectManifest, ProjectAuthoringError> {
    let source_index = SourceIndex::build(&sources)?;
    let source_revision = source_index.revision().clone();
    let manifest_digest = crate::manifest_digest(&manifest)?;
    Ok(AuthoredProjectManifest {
        manifest,
        root_module,
        sources,
        source_revision,
        source_index,
        manifest_digest,
    })
}
