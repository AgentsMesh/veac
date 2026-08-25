use std::path::Path;

use veac_project::ProjectPath;

use crate::{BuildError, BuildResult, ProjectFileSnapshot, ProjectSourceGraphRevision};

pub(super) fn capture(
    root: &Path,
    path: &ProjectPath,
    packages: &super::ProjectPackageSet,
) -> BuildResult<(ProjectFileSnapshot, ProjectSourceGraphRevision)> {
    super::checked_file(root, path)?;
    packages.revalidate()?;
    let (project, entry) =
        veac_lang::program::FileSystemLoader::for_root_entry(root, Path::new(path.as_str()))
            .map_err(|error| {
                BuildError::invalid(format!("cannot load evidence source graph: {error}"))
            })?;
    let loader = packages.loader(project)?;
    let authored = veac_evidence::build_evidence_with_loader(entry, &loader).map_err(|error| {
        BuildError::invalid(format!(
            "project evidence source graph {} is invalid: {error}",
            path.as_str()
        ))
    })?;
    packages.revalidate()?;
    let source = super::graph_root_source(&authored.sources, &authored.root_module, "evidence")?;
    let entry = ProjectFileSnapshot {
        path: path.as_str().to_owned(),
        content: veac_artifact::ContentDigest::sha256(source.as_bytes()),
        size_bytes: source.len() as u64,
    };
    let authored_module_count = super::graph_module_count(authored.sources.len(), "evidence")?;
    let authored_modules = authored.sources.keys().cloned().collect();
    Ok((
        entry,
        ProjectSourceGraphRevision {
            root_module: authored.root_module,
            authored_source_graph_sha256: authored.source_revision.source_graph_sha256,
            complete_source_graph_sha256: authored
                .complete_source_graph_revision
                .sha256()
                .to_owned(),
            authored_module_count,
            authored_modules,
        },
    ))
}
