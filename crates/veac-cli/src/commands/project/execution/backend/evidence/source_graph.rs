use std::path::Path;

use veac_build::{ProjectBackendError, ProjectFileSnapshot, ProjectSourceGraphRevision};

use super::super::{failed, inputs, ProjectResultExt};

pub(super) fn prepare(
    root: &Path,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<veac_evidence::AuthoredEvidenceSuite, ProjectBackendError> {
    let path = root.join(&entry.path);
    inputs::material::verify_snapshot(&path, entry)?;
    let authored = veac_evidence::build_evidence_root_path(root, Path::new(&entry.path))
        .project_context("evidence source preparation failed")?;
    verify_authored(&authored, entry, expected)?;
    Ok(authored)
}

pub(super) fn verify(
    root: &Path,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<(), ProjectBackendError> {
    prepare(root, entry, expected).map(|_| ())
}

fn verify_authored(
    authored: &veac_evidence::AuthoredEvidenceSuite,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<(), ProjectBackendError> {
    if authored.root_module != expected.root_module {
        return Err(failed("evidence source graph root module changed"));
    }
    let Some(source) = authored.sources.get(&authored.root_module) else {
        return Err(failed("evidence source graph omitted its root module"));
    };
    if veac_artifact::ContentDigest::sha256(source.as_bytes()) != entry.content
        || u64::try_from(source.len()).ok() != Some(entry.size_bytes)
    {
        return Err(failed("evidence source entry changed"));
    }
    let modules = authored.sources.keys().cloned().collect::<Vec<_>>();
    if u32::try_from(modules.len()).ok() != Some(expected.module_count)
        || modules.as_slice() != expected.modules.as_slice()
        || authored.source_revision.source_graph_sha256 != expected.source_graph_sha256
    {
        return Err(failed("evidence source graph revision changed"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "source_graph/tests.rs"]
mod tests;
