use veac_lang::program::{PreparedSourceGraph, SourceIndex};

use super::{AuthoredProjectManifest, ProjectAuthoringError};
use crate::ProjectManifestV1;

pub(super) fn finish(
    manifest: ProjectManifestV1,
    graph: &PreparedSourceGraph,
) -> Result<AuthoredProjectManifest, ProjectAuthoringError> {
    let source_index = SourceIndex::build(graph)?;
    let source_revision = source_index.revision().clone();
    let manifest_digest = crate::manifest_digest(&manifest)?;
    Ok(AuthoredProjectManifest {
        manifest,
        root_module: graph.root_module().to_owned(),
        sources: graph.project_sources(),
        source_revision,
        complete_source_graph_revision: graph.complete_revision(),
        source_index,
        manifest_digest,
    })
}
