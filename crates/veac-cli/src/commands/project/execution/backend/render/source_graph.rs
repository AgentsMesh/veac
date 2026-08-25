use std::path::Path;

use veac_build::{ProjectBackendError, ProjectFileSnapshot, ProjectSourceGraphRevision};
use veac_lang::program::ExecutableBuild;

use super::super::{failed, inputs};

pub(super) fn prepare(
    root: &Path,
    packages: &veac_build::ProjectPackageSet,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<ExecutableBuild, ProjectBackendError> {
    let path = root.join(&entry.path);
    inputs::material::verify_snapshot(&path, entry)?;
    packages
        .revalidate()
        .map_err(|error| failed(format!("target package verification failed: {error}")))?;
    let (project, source) = veac_lang::program::FileSystemLoader::for_entry(&path)
        .map_err(|error| failed(format!("target source loading failed: {error}")))?;
    let loader = packages
        .loader(project)
        .map_err(|error| failed(format!("target package loading failed: {error}")))?;
    let prepared = veac_lang::program::prepare_with_loader(source, &loader)
        .map_err(|error| failed(format!("target source preparation failed: {error}")))?;
    packages
        .revalidate()
        .map_err(|error| failed(format!("target package verification failed: {error}")))?;
    verify_prepared(&prepared, entry, expected)?;
    Ok(prepared)
}

pub(super) fn verify(
    root: &Path,
    packages: &veac_build::ProjectPackageSet,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<(), ProjectBackendError> {
    prepare(root, packages, entry, expected).map(|_| ())
}

fn verify_prepared(
    prepared: &ExecutableBuild,
    entry: &ProjectFileSnapshot,
    expected: &ProjectSourceGraphRevision,
) -> Result<(), ProjectBackendError> {
    if prepared.root_module() != expected.root_module {
        return Err(failed("target source graph root module changed"));
    }
    let source = match prepared.sources().get(prepared.root_module()) {
        Some(source) => source,
        None => return Err(failed("target source graph omitted its root module")),
    };
    if veac_artifact::ContentDigest::sha256(source.as_bytes()) != entry.content
        || u64::try_from(source.len()).ok() != Some(entry.size_bytes)
    {
        return Err(failed("target source entry changed during preparation"));
    }
    let authored = prepared.source_graph().project_sources();
    let modules = authored.keys().cloned().collect::<Vec<_>>();
    if u32::try_from(modules.len()).ok() != Some(expected.authored_module_count)
        || modules.as_slice() != expected.authored_modules.as_slice()
    {
        return Err(failed("target source graph module inventory changed"));
    }
    let modules = authored
        .iter()
        .map(|(path, source)| veac_lang::source_edit::SourceModule::utf8(path, source))
        .collect::<Vec<_>>();
    let revision = veac_lang::source_edit::source_graph_revision(&modules)
        .map_err(|error| failed(format!("target source revision failed: {error}")))?;
    if revision.source_graph_sha256 != expected.authored_source_graph_sha256 {
        return Err(failed("target authored source graph revision changed"));
    }
    if prepared.source_graph().complete_revision().sha256() != expected.complete_source_graph_sha256
    {
        return Err(failed("target complete source graph revision changed"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "source_graph/tests.rs"]
mod tests;
