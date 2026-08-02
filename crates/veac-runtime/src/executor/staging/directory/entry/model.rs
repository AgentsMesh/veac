use rustix::fs::AtFlags;
use serde::{Deserialize, Serialize};

use crate::RuntimeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "node_type", rename_all = "snake_case", deny_unknown_fields)]
pub(in crate::executor) enum EntryIdentity {
    Regular {
        device: u64,
        inode: u64,
        size_bytes: u64,
    },
    Directory {
        device: u64,
        inode: u64,
    },
}

impl EntryIdentity {
    pub fn size_bytes(self) -> Option<u64> {
        match self {
            Self::Regular { size_bytes, .. } => Some(size_bytes),
            Self::Directory { .. } => None,
        }
    }

    pub(super) fn remove_flags(self) -> AtFlags {
        match self {
            Self::Regular { .. } => AtFlags::empty(),
            Self::Directory { .. } => AtFlags::REMOVEDIR,
        }
    }

    pub(in crate::executor) fn same_kind(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Regular { .. }, Self::Regular { .. })
                | (Self::Directory { .. }, Self::Directory { .. })
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::executor) enum EntryState {
    Missing,
    Regular(EntryIdentity),
    Directory(EntryIdentity),
}

impl EntryState {
    pub(super) fn from_identity(identity: EntryIdentity) -> Self {
        match identity {
            EntryIdentity::Regular { .. } => Self::Regular(identity),
            EntryIdentity::Directory { .. } => Self::Directory(identity),
        }
    }

    pub(super) fn matches(self, expected: EntryIdentity) -> bool {
        matches!(self, Self::Regular(actual) | Self::Directory(actual) if actual == expected)
    }
}

#[derive(Debug)]
pub(in crate::executor) struct RenameFailure {
    pub error: RuntimeError,
    pub crossed_commit: bool,
}
