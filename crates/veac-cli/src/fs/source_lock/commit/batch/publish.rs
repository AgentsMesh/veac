use std::path::Path;

use super::{guard, path, ExpectedTarget, Prepared, SourceGraphLock, SourceModuleGuard, Staged};
use crate::error::{CliError, CliResult};

pub(super) struct Context<'a, 'guard> {
    lock: &'a SourceGraphLock,
    root: &'a Path,
    prepared: &'a [Prepared],
    guarded: &'a [guard::Guarded],
    guards: &'a [SourceModuleGuard<'guard>],
}

impl<'a, 'guard> Context<'a, 'guard> {
    pub(super) fn new(
        lock: &'a SourceGraphLock,
        root: &'a Path,
        prepared: &'a [Prepared],
        guarded: &'a [guard::Guarded],
        guards: &'a [SourceModuleGuard<'guard>],
    ) -> Self {
        Self {
            lock,
            root,
            prepared,
            guarded,
            guards,
        }
    }
}

pub(super) fn run(
    context: Context<'_, '_>,
    replacements: Vec<Staged<'_>>,
    rollbacks: Vec<Staged<'_>>,
    after_publish: impl FnOnce() -> CliResult,
) -> CliResult {
    let published_targets = replacements
        .iter()
        .map(Staged::expected_target)
        .collect::<Vec<_>>();
    let mut rollbacks = rollbacks.into_iter().map(Some).collect::<Vec<_>>();
    let mut published = 0usize;
    for (index, (staged, item)) in replacements.into_iter().zip(context.prepared).enumerate() {
        let expected = ExpectedTarget::from_target(&item.original);
        if let Err(error) = staged.publish(&item.parent, &expected, &item.label) {
            let uncertain = error.diagnostics()[0].code == "WRITE_COMMIT_UNCERTAIN";
            return rollback(
                context.prepared,
                &published_targets,
                &mut rollbacks,
                published + usize::from(uncertain),
                error,
            );
        }
        published = index + 1;
    }
    let validation = verify(&context)
        .and_then(|()| after_publish())
        .and_then(|()| verify(&context));
    if let Err(error) = validation {
        return rollback(
            context.prepared,
            &published_targets,
            &mut rollbacks,
            published,
            error,
        );
    }
    Ok(())
}

fn verify(context: &Context<'_, '_>) -> CliResult {
    context.lock.revalidate(context.root)?;
    for item in context.prepared {
        path::require_parent(
            &context.lock.directory,
            &item.module,
            item.parent.identity,
            &item.label,
        )?;
    }
    guard::verify(context.lock, context.guarded, context.guards)
}

fn rollback(
    prepared: &[Prepared],
    published_targets: &[ExpectedTarget],
    rollbacks: &mut [Option<Staged<'_>>],
    published: usize,
    original: CliError,
) -> CliResult {
    let mut failures = Vec::new();
    for index in (0..published).rev() {
        let staged = rollbacks[index].take().expect("rollback stage is present");
        if let Err(error) = staged.publish(
            &prepared[index].parent,
            &published_targets[index],
            &prepared[index].label,
        ) {
            failures.push(error.to_string());
        }
    }
    if failures.is_empty() {
        Err(original)
    } else {
        Err(CliError::new(
            "WRITE_COMMIT_UNCERTAIN",
            format!(
                "{original}; batch rollback failures: {}",
                failures.join("; ")
            ),
        ))
    }
}
