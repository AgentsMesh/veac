use std::collections::BTreeMap;

pub(super) fn index(sources: &BTreeMap<String, String>) -> super::SourceIndex {
    let root = sources
        .keys()
        .next()
        .expect("test source graph is nonempty");
    let authorities = sources
        .keys()
        .map(|id| (id.clone(), crate::program::SourceAuthority::Project))
        .collect();
    let graph = crate::program::PreparedSourceGraph::new(
        root.clone(),
        sources.clone(),
        authorities,
        BTreeMap::new(),
    );
    super::SourceIndex::build(&graph).unwrap()
}

pub(super) fn revision(index: &super::SourceIndex) -> crate::source_edit::SourceRevision {
    index.bound_revision().unwrap_or_else(|| {
        crate::source_edit::SourceRevision::new(index.revision(), &"f".repeat(64))
    })
}
