use std::collections::BTreeMap;

use super::{LoadedSource, SourceAuthority, SourceLoader};

mod revision;

pub use revision::PreparedSourceGraphRevision;

/// Immutable source bytes, import routes, and edit authority captured together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedSourceGraph {
    root_module: String,
    sources: BTreeMap<String, String>,
    authorities: BTreeMap<String, SourceAuthority>,
    resolutions: BTreeMap<(String, String), String>,
}

impl PreparedSourceGraph {
    pub(crate) fn new(
        root_module: String,
        sources: BTreeMap<String, String>,
        authorities: BTreeMap<String, SourceAuthority>,
        resolutions: BTreeMap<(String, String), String>,
    ) -> Self {
        debug_assert!(sources.contains_key(&root_module));
        debug_assert!(sources.keys().all(|id| authorities.contains_key(id)));
        Self {
            root_module,
            sources,
            authorities,
            resolutions,
        }
    }

    pub fn root_module(&self) -> &str {
        &self.root_module
    }

    pub fn sources(&self) -> &BTreeMap<String, String> {
        &self.sources
    }

    pub fn authority(&self, source_id: &str) -> SourceAuthority {
        self.authorities
            .get(source_id)
            .copied()
            .unwrap_or(SourceAuthority::ReadOnlyDependency)
    }

    pub fn project_sources(&self) -> BTreeMap<String, String> {
        self.sources
            .iter()
            .filter(|(id, _)| self.authority(id) == SourceAuthority::Project)
            .map(|(id, source)| (id.clone(), source.clone()))
            .collect()
    }

    pub(crate) fn contains_source(&self, source_id: &str) -> bool {
        self.sources.contains_key(source_id)
    }

    pub(crate) fn root(&self) -> LoadedSource {
        LoadedSource {
            id: self.root_module.clone(),
            source: self.sources[&self.root_module].clone(),
        }
    }

    pub(crate) fn resolutions(&self) -> &BTreeMap<(String, String), String> {
        &self.resolutions
    }

    pub(crate) fn resolved_id(&self, importer: &str, requested: &str) -> Option<&str> {
        self.resolutions
            .get(&(importer.to_owned(), requested.to_owned()))
            .map(String::as_str)
    }
}

impl SourceLoader for PreparedSourceGraph {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = self
            .resolved_id(importer, requested)
            .ok_or_else(|| format!("source graph has no resolution for `{requested}`"))?;
        Ok(LoadedSource {
            id: id.to_owned(),
            source: self.sources[id].clone(),
        })
    }

    fn authority(&self, source_id: &str) -> SourceAuthority {
        PreparedSourceGraph::authority(self, source_id)
    }
}

#[cfg(test)]
#[path = "prepared_source_graph/revision_tests.rs"]
mod revision_tests;
