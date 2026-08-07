use std::collections::BTreeMap;
use std::path::Path;

use super::path::{confined_request, normalize};
use super::{LoadedSource, SourceLoader};

#[derive(Debug, Clone, Default)]
pub(crate) struct MemoryLoader {
    pub sources: BTreeMap<String, String>,
}

impl MemoryLoader {
    pub(crate) fn new(sources: BTreeMap<String, String>) -> Self {
        Self { sources }
    }
}

impl SourceLoader for MemoryLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let requested = confined_request(requested)?;
        let parent = Path::new(importer)
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let id = normalize(&parent.join(&requested))?;
        self.sources
            .get(&id)
            .cloned()
            .map(|source| LoadedSource { id, source })
            .ok_or_else(|| {
                format!(
                    "module `{}` imported by `{importer}` was not found",
                    requested.display()
                )
            })
    }
}

#[cfg(test)]
#[path = "memory/tests.rs"]
mod tests;
