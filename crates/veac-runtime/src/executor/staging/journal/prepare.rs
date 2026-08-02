use std::path::{Component, Path, PathBuf};

use super::super::directory::{Directory, EntryIdentity, EntryState};
use super::super::StagedOutput;
use super::{
    storage, validation, Journal, JournalEntry, JournalState, JOURNAL_NAME, SCHEMA_VERSION,
};
use crate::RuntimeError;

pub(in crate::executor::staging) fn prepare(
    stage: &Directory,
    output: &Directory,
    outputs: &[StagedOutput],
    identities: &[EntryIdentity],
    stale: &[PathBuf],
) -> Result<Journal, RuntimeError> {
    let parent = outputs[0].target().parent().unwrap_or(Path::new("."));
    let mut values = outputs
        .iter()
        .zip(identities)
        .map(|(output, identity)| {
            names(parent, output.target(), Some(output.source()))
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
                original: original(
                    output,
                    &target,
                    source_identity.unwrap_or_else(stale_identity),
                )?,
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
    validation::validate(&journal)?;
    if !stage.missing(JOURNAL_NAME)? {
        return validation::invalid("commit journal already exists before prepare");
    }
    storage::persist(stage, &journal).map_err(|failure| failure.error)?;
    Ok(journal)
}

fn names(
    parent: &Path,
    target: &Path,
    source: Option<&Path>,
) -> Result<(String, Option<String>), RuntimeError> {
    if target.parent().unwrap_or(Path::new(".")) != parent {
        return validation::invalid("commit journal targets must share one directory");
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

fn original(
    output: &Directory,
    name: &str,
    replacement: EntryIdentity,
) -> Result<Option<EntryIdentity>, RuntimeError> {
    let state = output.state(name).map_err(|error| {
        RuntimeError::new(format!("refusing to replace unsafe output {name}: {error}"))
    })?;
    match state {
        EntryState::Regular(identity) | EntryState::Directory(identity)
            if identity.same_kind(replacement) =>
        {
            Ok(Some(identity))
        }
        EntryState::Regular(_) => Err(RuntimeError::new(format!(
            "refusing to replace unsafe output {name}: target must be a directory"
        ))),
        EntryState::Directory(_) => Err(RuntimeError::new(format!(
            "refusing to replace unsafe output {name}: target must be an exclusively linked regular file"
        ))),
        EntryState::Missing => Ok(None),
    }
}

fn stale_identity() -> EntryIdentity {
    EntryIdentity::Regular {
        device: 0,
        inode: 0,
        size_bytes: 0,
    }
}
