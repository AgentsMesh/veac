use std::collections::BTreeMap;

use crate::source_edit::{source_graph_revision, SourceEditError, SourceModule, SourceRevision};

pub(super) fn calculate(
    sources: &BTreeMap<String, String>,
) -> Result<SourceRevision, SourceEditError> {
    let modules = sources
        .iter()
        .map(|(path, source)| SourceModule::utf8(path, source))
        .collect::<Vec<_>>();
    source_graph_revision(&modules)
}
