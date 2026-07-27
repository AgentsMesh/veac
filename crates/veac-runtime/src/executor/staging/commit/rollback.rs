use super::super::directory::{Directory, EntryIdentity};
use crate::RuntimeError;

pub(in crate::executor) trait Operations {
    fn remove(
        &self,
        output: &Directory,
        name: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError>;

    fn restore(
        &self,
        backup: &Directory,
        source: &str,
        output: &Directory,
        target: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError>;
}

pub(super) struct LiveOperations;

impl Operations for LiveOperations {
    fn remove(
        &self,
        output: &Directory,
        name: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError> {
        output.remove_bound(name, expected)
    }

    fn restore(
        &self,
        backup: &Directory,
        source: &str,
        output: &Directory,
        target: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError> {
        backup
            .rename_bound_to(source, expected, output, target)
            .map_err(|failure| failure.error)
    }
}

pub(super) fn apply<O: Operations>(
    output: &Directory,
    backup: &Directory,
    installed: &[(String, EntryIdentity)],
    backups: &[(String, String, EntryIdentity)],
    operations: &O,
) -> Result<(), RuntimeError> {
    let mut failures = Vec::new();
    for (name, identity) in installed.iter().rev() {
        if let Err(error) = operations.remove(output, name, *identity) {
            failures.push(error.to_string());
        }
    }
    for (source, target, identity) in backups.iter().rev() {
        if let Err(error) = operations.restore(backup, source, output, target, *identity) {
            failures.push(error.to_string());
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(RuntimeError::new(failures.join("; ")))
    }
}
