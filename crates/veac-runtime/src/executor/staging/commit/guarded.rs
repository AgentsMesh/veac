use super::super::directory::{Directory, EntryIdentity, EntryState};
use crate::RuntimeError;

pub(super) fn verify_original(
    output: &Directory,
    name: &str,
    expected: Option<EntryIdentity>,
) -> Result<(), RuntimeError> {
    match (output.state(name)?, expected) {
        (EntryState::Regular(actual), Some(expected)) if actual == expected => Ok(()),
        (EntryState::Missing, None) => Ok(()),
        _ => Err(RuntimeError::new(format!(
            "refusing to replace unsafe or concurrently changed output {name}"
        ))),
    }
}

pub(super) fn sync(
    output: &Directory,
    installed: &[(String, EntryIdentity)],
    guard: &mut impl FnMut() -> bool,
) -> Result<(), RuntimeError> {
    for (name, identity) in installed {
        active(guard)?;
        output.sync_bound(name, *identity)?;
        active(guard)?;
    }
    active(guard)?;
    output.sync()
}

pub(super) fn active(guard: &mut impl FnMut() -> bool) -> Result<(), RuntimeError> {
    if guard() {
        Ok(())
    } else {
        Err(RuntimeError::resource_limit(
            "output commit exceeded the backend task deadline",
        ))
    }
}
