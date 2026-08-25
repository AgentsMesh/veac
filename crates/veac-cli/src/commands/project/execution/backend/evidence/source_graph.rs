use std::path::Path;

use veac_build::{ProjectBackendError, ProjectFileSnapshot, ProjectSourceGraphRevision};

use super::super::{failed, inputs, ProjectResultExt};

pub(super) fn prepare(
    root: &Path,
    packages: &veac_build::ProjectPackageSet,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<veac_evidence::AuthoredEvidenceSuite, ProjectBackendError> {
    let path = root.join(&entry.path);
    inputs::material::verify_snapshot(&path, entry)?;
    packages
        .revalidate()
        .map_err(|error| failed(format!("evidence package verification failed: {error}")))?;
    let (project, source) =
        veac_lang::program::FileSystemLoader::for_root_entry(root, Path::new(&entry.path))
            .map_err(|error| failed(format!("evidence source loading failed: {error}")))?;
    let loader = packages
        .loader(project)
        .map_err(|error| failed(format!("evidence package loading failed: {error}")))?;
    let authored = veac_evidence::build_evidence_with_loader(source, &loader)
        .project_context("evidence source preparation failed")?;
    packages
        .revalidate()
        .map_err(|error| failed(format!("evidence package verification failed: {error}")))?;
    verify_authored(&authored, entry, expected)?;
    Ok(authored)
}

pub(super) fn verify(
    root: &Path,
    packages: &veac_build::ProjectPackageSet,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<(), ProjectBackendError> {
    prepare(root, packages, entry, expected).map(|_| ())
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
    if u32::try_from(modules.len()).ok() != Some(expected.authored_module_count)
        || modules.as_slice() != expected.authored_modules.as_slice()
        || authored.source_revision.source_graph_sha256 != expected.authored_source_graph_sha256
    {
        return Err(failed("evidence authored source graph revision changed"));
    }
    if authored.complete_source_graph_revision.sha256() != expected.complete_source_graph_sha256 {
        return Err(failed("evidence complete source graph revision changed"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "source_graph/tests.rs"]
mod tests;
