use serde::{Deserialize, Serialize};

use super::super::directory::EntryIdentity;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::executor::staging) struct Journal {
    pub(super) schema_version: u32,
    pub state: JournalState,
    pub entries: Vec<JournalEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::executor::staging) enum JournalState {
    Prepared,
    Committed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::executor::staging) struct JournalEntry {
    pub target: String,
    pub source: Option<String>,
    pub source_identity: Option<EntryIdentity>,
    pub original: Option<EntryIdentity>,
}
