use veac_lang::program::{PreparedSourceGraph, SourceIndex};

use super::{AuthoredEvidenceSuite, EvidenceAuthoringError};
use crate::EvidenceSuiteV1;

pub(super) fn finish(
    suite: EvidenceSuiteV1,
    graph: &PreparedSourceGraph,
) -> Result<AuthoredEvidenceSuite, EvidenceAuthoringError> {
    let source_index = SourceIndex::build(graph)?;
    let source_revision = source_index.revision().clone();
    let suite_sha256 = crate::sha256_hex(&crate::canonical_json(&suite)?);
    Ok(AuthoredEvidenceSuite {
        suite,
        root_module: graph.root_module().to_owned(),
        sources: graph.project_sources(),
        source_revision,
        complete_source_graph_revision: graph.complete_revision(),
        source_index,
        suite_sha256,
    })
}
