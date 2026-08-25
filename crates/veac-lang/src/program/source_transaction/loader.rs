use std::collections::BTreeMap;

use super::super::loader::MemoryLoader;
use super::super::{LoadedSource, PreparedSourceGraph, SourceAuthority, SourceLoader};

pub(super) struct CandidateLoader<'a> {
    overlay: MemoryLoader,
    prepared: &'a PreparedSourceGraph,
    fallback: &'a dyn SourceLoader,
}

impl<'a> CandidateLoader<'a> {
    pub(super) fn new(
        overlay: BTreeMap<String, String>,
        prepared: &'a PreparedSourceGraph,
        fallback: &'a dyn SourceLoader,
    ) -> Self {
        Self {
            overlay: MemoryLoader::new(overlay),
            prepared,
            fallback,
        }
    }

    fn validate_route(
        &self,
        importer: &str,
        requested: &str,
        loaded: &LoadedSource,
    ) -> Result<(), String> {
        let Some(expected) = self.prepared.resolved_id(importer, requested) else {
            return Ok(());
        };
        if loaded.id == expected {
            return Ok(());
        }
        Err(format!(
            "prepared import `{requested}` changed source ID from `{expected}` to `{}`",
            loaded.id
        ))
    }

    fn validate_bytes(&self, loaded: &LoadedSource) -> Result<(), String> {
        let Some(expected) = self.prepared.sources().get(&loaded.id) else {
            return Ok(());
        };
        if expected == &loaded.source {
            return Ok(());
        }
        Err(format!(
            "source `{}` changed bytes since the prepared graph",
            loaded.id
        ))
    }
}

impl SourceLoader for CandidateLoader<'_> {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let mut loaded = self.fallback.load(importer, requested)?;
        self.validate_route(importer, requested, &loaded)?;
        self.validate_bytes(&loaded)?;
        if let Some(source) = self.overlay.sources.get(&loaded.id) {
            loaded.source.clone_from(source);
        }
        Ok(loaded)
    }

    fn authority(&self, source_id: &str) -> SourceAuthority {
        if self.prepared.contains_source(source_id) {
            self.prepared.authority(source_id)
        } else {
            self.fallback.authority(source_id)
        }
    }
}

pub(super) struct ReplayLoader<'a> {
    overlay: MemoryLoader,
    fallback: &'a dyn SourceLoader,
}

impl<'a> ReplayLoader<'a> {
    pub(super) fn new(overlay: BTreeMap<String, String>, fallback: &'a dyn SourceLoader) -> Self {
        Self {
            overlay: MemoryLoader::new(overlay),
            fallback,
        }
    }
}

impl SourceLoader for ReplayLoader<'_> {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let mut loaded = self.fallback.load(importer, requested)?;
        if let Some(source) = self.overlay.sources.get(&loaded.id) {
            loaded.source.clone_from(source);
        }
        Ok(loaded)
    }

    fn authority(&self, source_id: &str) -> SourceAuthority {
        if self.overlay.sources.contains_key(source_id) {
            SourceAuthority::Project
        } else {
            self.fallback.authority(source_id)
        }
    }
}
