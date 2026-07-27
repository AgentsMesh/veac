use std::time::Instant;

use super::directory::{Directory, EntryIdentity};
use super::journal;
use crate::RuntimeError;

mod context;
mod failure;
mod guarded;
mod observer;
mod rollback;
mod validation;

pub(in crate::executor) use failure::CommitFailure;
use guarded::{active, sync, verify_original};
use observer::{LiveObserver, Observer};

pub(in crate::executor) use context::CommitContext;

#[cfg(test)]
pub(in crate::executor) use observer::Observer as CommitObserver;

#[cfg(test)]
pub(in crate::executor) use rollback::Operations as RollbackOperations;

pub(super) fn apply_locked(
    context: CommitContext<'_>,
    deadline: Instant,
) -> Result<(), CommitFailure> {
    apply_with(
        context,
        || Instant::now() < deadline,
        &rollback::LiveOperations,
    )
}

pub(in crate::executor) fn apply_with<O: rollback::Operations>(
    context: CommitContext<'_>,
    guard: impl FnMut() -> bool,
    rollback: &O,
) -> Result<(), CommitFailure> {
    apply_observed(context, guard, rollback, &LiveObserver)
}

pub(in crate::executor) fn apply_observed<R: rollback::Operations, O: Observer>(
    context: CommitContext<'_>,
    mut guard: impl FnMut() -> bool,
    rollback: &R,
    observer: &O,
) -> Result<(), CommitFailure> {
    let CommitContext {
        staging,
        stage,
        output,
        files,
        stale,
        allow_empty,
    } = context;
    active(&mut guard)?;
    let identities = validation::validate(staging, stage, files, stale, allow_empty)?;
    active(&mut guard)?;
    let backup = stage.create_child("backups")?;
    active(&mut guard)?;
    let mut transaction = journal::prepare(stage, output, files, &identities, stale)?;
    active(&mut guard)?;
    let mut backups = Vec::new();
    for (index, entry) in transaction.entries.iter().enumerate() {
        if let Err(error) = active(&mut guard) {
            return Err(failed(error, output, &backup, &[], &backups, rollback));
        }
        if let Err(error) = verify_original(output, &entry.target, entry.original) {
            return Err(failed(error, output, &backup, &[], &backups, rollback));
        }
        let Some(identity) = entry.original else {
            continue;
        };
        let name = index.to_string();
        if let Err(failure) =
            output.rename_bound_to_with(&entry.target, identity, &backup, &name, || {
                observer.before_backup(&entry.target)
            })
        {
            if failure.crossed_commit {
                backups.push((name, entry.target.clone(), identity));
            }
            return Err(failed(
                failure.error,
                output,
                &backup,
                &[],
                &backups,
                rollback,
            ));
        }
        backups.push((name, entry.target.clone(), identity));
        if let Err(error) = active(&mut guard) {
            return Err(failed(error, output, &backup, &[], &backups, rollback));
        }
    }
    let mut installed = Vec::new();
    for entry in transaction
        .entries
        .iter()
        .filter(|entry| entry.source.is_some())
    {
        if let Err(error) = active(&mut guard) {
            return Err(failed(
                error, output, &backup, &installed, &backups, rollback,
            ));
        }
        let source = entry.source.as_deref().unwrap();
        let identity = entry
            .source_identity
            .expect("validated journal source has an identity");
        if let Err(failure) =
            stage.rename_bound_to_with(source, identity, output, &entry.target, || {
                observer.before_install(source, &entry.target)
            })
        {
            if failure.crossed_commit {
                installed.push((entry.target.clone(), identity));
            }
            return Err(failed(
                failure.error,
                output,
                &backup,
                &installed,
                &backups,
                rollback,
            ));
        }
        installed.push((entry.target.clone(), identity));
        if let Err(error) = active(&mut guard) {
            return Err(failed(
                error, output, &backup, &installed, &backups, rollback,
            ));
        }
    }
    let committed = sync(output, &installed, &mut guard)
        .and_then(|_| active(&mut guard))
        .and_then(|_| journal::commit(stage, &mut transaction));
    if let Err(error) = committed {
        return Err(failed(
            error, output, &backup, &installed, &backups, rollback,
        ));
    }
    Ok(())
}

fn failed<O: rollback::Operations>(
    error: RuntimeError,
    output: &Directory,
    backup: &Directory,
    installed: &[(String, EntryIdentity)],
    backups: &[(String, String, EntryIdentity)],
    operations: &O,
) -> CommitFailure {
    match rollback::apply(output, backup, installed, backups, operations) {
        Ok(()) => CommitFailure::direct(error),
        Err(rollback) => CommitFailure::rollback(error, rollback),
    }
}
