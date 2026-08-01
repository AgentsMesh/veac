use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use veac_artifact::ContentDigest;
use veac_codegen::emitter::BackendPhase;

use super::model::RuntimeBundle;
use crate::RuntimeError;

pub(super) mod filter;
mod pass;
mod paths;
pub(in crate::executor) mod pattern;
mod requirement;
mod resource;
mod store;
mod task;

#[cfg(test)]
mod tests;

pub(super) struct ValidatedContract {
    pub plan: ContentDigest,
    pub resources: ContentDigest,
    pub output_parent: PathBuf,
    resource_paths: BTreeSet<PathBuf>,
}

pub(super) fn validate_until(
    bundle: &RuntimeBundle,
    deadline: Instant,
) -> Result<ValidatedContract, RuntimeError> {
    validate_while(bundle, || Instant::now() < deadline)
}

fn validate_while(
    bundle: &RuntimeBundle,
    mut guard: impl FnMut() -> bool,
) -> Result<ValidatedContract, RuntimeError> {
    active(&mut guard)?;
    let plan = bundle.plan_identity.clone();
    plan.validate()
        .map_err(|error| RuntimeError::new(format!("invalid render plan hash: {error}")))?;
    if bundle.tasks.is_empty() {
        return invalid("backend bundle must contain at least one task");
    }
    let resources = resource::validate_while(bundle, &mut guard)?;
    active(&mut guard)?;
    filter::validate(bundle)?;
    active(&mut guard)?;
    let output_parent = paths::validate(bundle, &resources.paths)?;
    let mut phases: BTreeMap<String, Vec<BackendPhase>> = BTreeMap::new();
    for task in &bundle.tasks {
        active(&mut guard)?;
        task::validate(task)?;
        phases
            .entry(task.deliverable_id.to_string())
            .or_default()
            .push(task.phase);
    }
    for values in phases.values() {
        active(&mut guard)?;
        if values.as_slice() != [BackendPhase::Single]
            && values.as_slice() != [BackendPhase::FirstPass, BackendPhase::SecondPass]
        {
            return invalid("deliverable tasks must be single or ordered first/second pass");
        }
    }
    pass::validate(bundle)?;
    active(&mut guard)?;
    requirement::validate(bundle, &phases)?;
    active(&mut guard)?;
    Ok(ValidatedContract {
        plan,
        resources: resources.fingerprint,
        output_parent,
        resource_paths: resources.paths,
    })
}

pub(super) fn validate_filesystem_paths_until(
    bundle: &RuntimeBundle,
    contract: &ValidatedContract,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    crate::executor::deadline::ensure_setup(deadline)?;
    paths::validate_filesystem(bundle, &contract.resource_paths)?;
    crate::executor::deadline::ensure_setup(deadline)
}

pub(super) fn validate_store(bundle: &RuntimeBundle, root: &Path) -> Result<(), RuntimeError> {
    store::validate(bundle, root)
}

pub(super) fn verify_resources_until(
    bundle: &RuntimeBundle,
    expected: &ContentDigest,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    resource::verify_until(bundle, expected, deadline)
}

pub(super) fn invalid<T>(message: &str) -> Result<T, RuntimeError> {
    Err(RuntimeError::new(format!(
        "invalid backend bundle: {message}"
    )))
}

fn active(guard: &mut impl FnMut() -> bool) -> Result<(), RuntimeError> {
    if guard() {
        Ok(())
    } else {
        Err(RuntimeError::resource_limit(
            "bundle contract validation exceeded its setup deadline",
        ))
    }
}
