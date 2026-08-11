use std::path::Path;

use veac_project::ProjectPath;

use crate::{BuildError, BuildResult, ProjectFileSnapshot, ProjectSourceGraphRevision};

pub(super) fn capture(
    root: &Path,
    path: &ProjectPath,
) -> BuildResult<(ProjectFileSnapshot, ProjectSourceGraphRevision)> {
    super::checked_file(root, path)?;
    let authored = veac_evidence::build_evidence_root_path(root, Path::new(path.as_str()))
        .map_err(|error| {
            BuildError::invalid(format!(
                "project evidence source graph {} is invalid: {error}",
                path.as_str()
            ))
        })?;
    let source = super::graph_root_source(&authored.sources, &authored.root_module, "evidence")?;
    let entry = ProjectFileSnapshot {
        path: path.as_str().to_owned(),
        content: veac_artifact::ContentDigest::sha256(source.as_bytes()),
        size_bytes: source.len() as u64,
    };
    let module_count = super::graph_module_count(authored.sources.len(), "evidence")?;
    let modules = authored.sources.keys().cloned().collect();
    Ok((
        entry,
        ProjectSourceGraphRevision {
            root_module: authored.root_module,
            source_graph_sha256: authored.source_revision.source_graph_sha256,
            module_count,
            modules,
        },
    ))
}
