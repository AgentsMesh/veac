use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use super::directory::{Directory, EntryIdentity, EntryState};
use super::StagedFile;
use crate::RuntimeError;

pub(super) const JOURNAL_NAME: &str = ".veac-commit.json";
pub(super) const TEMP_NAME: &str = ".veac-commit.tmp";
const SCHEMA_VERSION: u32 = 2;

mod storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Journal {
    schema_version: u32,
    pub state: JournalState,
    pub entries: Vec<JournalEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum JournalState {
    Prepared,
    Committed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalEntry {
    pub target: String,
    pub source: Option<String>,
    pub source_identity: Option<EntryIdentity>,
    pub original: Option<EntryIdentity>,
}

pub(super) fn prepare(
    stage: &Directory,
    output: &Directory,
    files: &[StagedFile],
    identities: &[EntryIdentity],
    stale: &[std::path::PathBuf],
) -> Result<Journal, RuntimeError> {
    let parent = files[0].target.parent().unwrap_or(Path::new("."));
    let mut values = files
        .iter()
        .zip(identities)
        .map(|(file, identity)| {
            names(parent, &file.target, Some(&file.source))
                .map(|(target, source)| (target, source, Some(*identity)))
        })
        .collect::<Result<Vec<_>, _>>()?;
    values.extend(
        stale
            .iter()
            .map(|target| {
                names(parent, target, None::<&Path>).map(|(target, source)| (target, source, None))
            })
            .collect::<Result<Vec<_>, _>>()?,
    );
    values.sort_by(|left, right| left.0.cmp(&right.0));
    let entries = values
        .into_iter()
        .map(|(target, source, source_identity)| {
            Ok(JournalEntry {
                original: original(output, &target)?,
                target,
                source,
                source_identity,
            })
        })
        .collect::<Result<Vec<_>, RuntimeError>>()?;
    let journal = Journal {
        schema_version: SCHEMA_VERSION,
        state: JournalState::Prepared,
        entries,
    };
    validate(&journal)?;
    if !stage.missing(JOURNAL_NAME)? {
        return invalid("commit journal already exists before prepare");
    }
    storage::persist(stage, &journal)?;
    Ok(journal)
}

pub(super) fn commit(stage: &Directory, journal: &mut Journal) -> Result<(), RuntimeError> {
    journal.state = JournalState::Committed;
    storage::persist(stage, journal)
}

pub(super) fn load_bound(stage: &Directory) -> Result<(Journal, EntryIdentity), RuntimeError> {
    let (bytes, identity) = storage::load(stage)?;
    let value: Journal = serde_json::from_slice(&bytes)
        .map_err(|error| RuntimeError::new(format!("invalid render commit journal: {error}")))?;
    validate(&value)?;
    Ok((value, identity))
}

fn names(
    parent: &Path,
    target: &Path,
    source: Option<&Path>,
) -> Result<(String, Option<String>), RuntimeError> {
    if target.parent().unwrap_or(Path::new(".")) != parent {
        return invalid("commit journal targets must share one directory");
    }
    Ok((component(target)?, source.map(component).transpose()?))
}

fn component(path: &Path) -> Result<String, RuntimeError> {
    let mut components = path.components();
    let value = match (components.next_back(), components.next_back()) {
        (Some(Component::Normal(value)), None) => value,
        _ => path
            .file_name()
            .ok_or_else(|| RuntimeError::new("path has no file name"))?,
    };
    value
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| RuntimeError::new("commit journal paths must be valid UTF-8"))
}

fn original(output: &Directory, name: &str) -> Result<Option<EntryIdentity>, RuntimeError> {
    let state = output.state(name).map_err(|error| {
        RuntimeError::new(format!("refusing to replace unsafe output {name}: {error}"))
    })?;
    match state {
        EntryState::Regular(identity) => Ok(Some(identity)),
        EntryState::Missing => Ok(None),
    }
}

fn validate(journal: &Journal) -> Result<(), RuntimeError> {
    if journal.schema_version != SCHEMA_VERSION || journal.entries.is_empty() {
        return invalid("unsupported or empty commit journal");
    }
    let mut previous = None;
    for entry in &journal.entries {
        if !safe_name(&entry.target)
            || entry
                .source
                .as_deref()
                .is_some_and(|value| !safe_name(value))
            || entry.source.is_some() != entry.source_identity.is_some()
            || previous.is_some_and(|value| value >= entry.target.as_str())
        {
            return invalid(
                "commit journal paths must be paired, sorted, unique, and single-component",
            );
        }
        previous = Some(entry.target.as_str());
    }
    Ok(())
}

fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value).components().count() == 1
        && matches!(
            Path::new(value).components().next(),
            Some(Component::Normal(_))
        )
}

fn invalid<T>(message: &str) -> Result<T, RuntimeError> {
    Err(invalid_error(message))
}

fn invalid_error(message: &str) -> RuntimeError {
    RuntimeError::new(format!("invalid render commit journal: {message}"))
}
