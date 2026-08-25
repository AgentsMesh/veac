use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use veac_lang::program::{LoadedSource, SourceLoader};

#[derive(Clone)]
pub(crate) struct MutableLoader {
    sources: Arc<Mutex<BTreeMap<String, String>>>,
}

impl MutableLoader {
    pub(crate) fn new(entries: impl IntoIterator<Item = (&'static str, &'static str)>) -> Self {
        Self {
            sources: Arc::new(Mutex::new(
                entries
                    .into_iter()
                    .map(|(id, source)| (id.into(), source.into()))
                    .collect(),
            )),
        }
    }

    pub(crate) fn replace(&self, id: &str, source: &str) {
        self.sources
            .lock()
            .expect("mutable loader lock poisoned")
            .insert(id.into(), source.into());
    }
}

impl SourceLoader for MutableLoader {
    fn load(&self, _importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let id = requested.trim_start_matches("./").to_owned();
        self.sources
            .lock()
            .expect("mutable loader lock poisoned")
            .get(&id)
            .cloned()
            .map(|source| LoadedSource { id, source })
            .ok_or_else(|| format!("missing module `{requested}`"))
    }

    fn authority(&self, _source_id: &str) -> veac_lang::program::SourceAuthority {
        veac_lang::program::SourceAuthority::Project
    }
}
