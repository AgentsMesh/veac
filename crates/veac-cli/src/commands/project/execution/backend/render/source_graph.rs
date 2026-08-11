use std::path::Path;

use veac_build::{ProjectBackendError, ProjectFileSnapshot, ProjectSourceGraphRevision};
use veac_lang::program::ExecutableBuild;

use super::super::{failed, inputs};

pub(super) fn prepare(
    root: &Path,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<ExecutableBuild, ProjectBackendError> {
    let path = root.join(&entry.path);
    inputs::material::verify_snapshot(&path, entry)?;
    let prepared = veac_lang::program::prepare_path(&path)
        .map_err(|error| failed(format!("target source preparation failed: {error}")))?;
    verify_prepared(&prepared, entry, expected)?;
    Ok(prepared)
}

pub(super) fn verify(
    root: &Path,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<(), ProjectBackendError> {
    prepare(root, entry, expected).map(|_| ())
}

fn verify_prepared(
    prepared: &ExecutableBuild,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<(), ProjectBackendError> {
    if prepared.root_module() != expected.root_module {
        return Err(failed("target source graph root module changed"));
    }
    let source = prepared
        .sources()
        .get(prepared.root_module())
        .ok_or_else(|| failed("target source graph omitted its root module"))?;
    if veac_artifact::ContentDigest::sha256(source.as_bytes()) != entry.content
        || u64::try_from(source.len()).ok() != Some(entry.size_bytes)
    {
        return Err(failed("target source entry changed during preparation"));
    }
    let modules = prepared.sources().keys().cloned().collect::<Vec<_>>();
    if u32::try_from(modules.len()).ok() != Some(expected.module_count)
        || modules.as_slice() != expected.modules.as_slice()
    {
        return Err(failed("target source graph module inventory changed"));
    }
    let modules = prepared
        .sources()
        .iter()
        .map(|(path, source)| veac_lang::source_edit::SourceModule::utf8(path, source))
        .collect::<Vec<_>>();
    let revision = veac_lang::source_edit::source_graph_revision(&modules)
        .map_err(|error| failed(format!("target source revision failed: {error}")))?;
    if revision.source_graph_sha256 != expected.source_graph_sha256 {
        return Err(failed("target source graph revision changed"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "source_graph/tests.rs"]
mod tests;
