use veac_artifact::{ArtifactStore, ContentDigest};

use crate::RuntimeError;

pub(in crate::executor) struct FreshCheckpoints<'a> {
    store: &'a ArtifactStore,
    keys: Vec<ContentDigest>,
}

impl<'a> FreshCheckpoints<'a> {
    pub(in crate::executor) fn new(store: &'a ArtifactStore) -> Self {
        Self {
            store,
            keys: Vec::new(),
        }
    }

    pub(in crate::executor) fn track(&mut self, key: ContentDigest) {
        self.keys.push(key);
    }

    pub(in crate::executor) fn rollback(self, error: RuntimeError) -> RuntimeError {
        let mut failures = Vec::new();
        for key in self.keys.iter().rev() {
            if let Err(cleanup) = self.store.remove(key) {
                failures.push(cleanup.to_string());
            }
        }
        if failures.is_empty() {
            error
        } else {
            RuntimeError {
                kind: error.kind,
                message: format!(
                    "{error}; cannot discard fresh render checkpoints: {}",
                    failures.join("; ")
                ),
            }
        }
    }
}
