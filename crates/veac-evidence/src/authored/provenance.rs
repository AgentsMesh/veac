use std::collections::BTreeMap;

use veac_lang::program::SourceIndex;

use super::{AuthoredEvidenceSuite, EvidenceAuthoringError, EVIDENCE_MODULE_ID};
use crate::EvidenceSuiteV1;

pub(super) fn authored_sources(resolved: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    resolved
        .iter()
        .filter(|(path, _)| path.as_str() != EVIDENCE_MODULE_ID)
        .map(|(path, source)| (path.clone(), source.clone()))
        .collect()
}

pub(super) fn finish(
    suite: EvidenceSuiteV1,
    root_module: String,
    sources: BTreeMap<String, String>,
) -> Result<AuthoredEvidenceSuite, EvidenceAuthoringError> {
    let source_index = SourceIndex::build(&sources)?;
    let source_revision = source_index.revision().clone();
    let suite_sha256 = crate::sha256_hex(&crate::canonical_json(&suite)?);
    Ok(AuthoredEvidenceSuite {
        suite,
        root_module,
        sources,
        source_revision,
        source_index,
        suite_sha256,
    })
}
