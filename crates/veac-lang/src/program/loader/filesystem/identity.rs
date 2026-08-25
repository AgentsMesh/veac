use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct FileIdentity {
    device: i128,
    inode: u128,
}

impl FileIdentity {
    pub(super) fn from_stat(value: &rustix::fs::Stat) -> Self {
        Self {
            device: i128::from(value.st_dev),
            inode: u128::from(value.st_ino),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct IdentityRegistry {
    paths: Arc<Mutex<BTreeMap<FileIdentity, String>>>,
}

impl IdentityRegistry {
    pub(super) fn register(&self, id: &str, identity: FileIdentity) -> Result<(), String> {
        let mut paths = self
            .paths
            .lock()
            .map_err(|_| "source identity registry is unavailable".to_owned())?;
        if let Some(existing) = paths.get(&identity) {
            if existing != id {
                return Err(format!(
                    "source `{id}` and `{existing}` refer to the same physical file"
                ));
            }
        } else {
            paths.insert(identity, id.to_owned());
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "identity/tests.rs"]
mod tests;
