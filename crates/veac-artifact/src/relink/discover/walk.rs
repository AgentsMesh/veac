use std::collections::BTreeSet;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, RelinkCandidate};

use super::platform::{BoundDirectory, BoundEntry};
use super::{DiscoveryBudget, DiscoveryHookPoint};

pub(super) fn visit(
    directory: &BoundDirectory,
    depth: usize,
    candidates: &mut Vec<RelinkCandidate>,
    seen: &mut BTreeSet<std::path::PathBuf>,
    budget: &mut DiscoveryBudget,
    hook: &mut impl FnMut(DiscoveryHookPoint, &std::path::Path),
) -> ArtifactResult<()> {
    if depth > budget.max_depth() {
        return limit("relink discovery exceeds the directory depth budget");
    }
    for name in directory.entries(budget)? {
        let path = directory.path().join(&name);
        hook(DiscoveryHookPoint::EntryEnumerated, &path);
        budget.check_deadline()?;
        match directory.open_entry(&name, budget)? {
            BoundEntry::Directory(child) => {
                let next = depth
                    .checked_add(1)
                    .ok_or_else(|| limit_error("relink discovery depth overflowed"))?;
                visit(&child, next, candidates, seen, budget, hook)?;
                let seal = child.seal(budget)?;
                directory.verify_entry(&name, &seal, budget)?;
            }
            BoundEntry::File(mut file) => {
                let path = file.path().to_owned();
                if seen.insert(path.clone()) {
                    let (verified, seal) = file.verify(budget)?;
                    hook(DiscoveryHookPoint::FileHashed, &path);
                    budget.check_deadline()?;
                    directory.verify_entry(&name, &seal, budget)?;
                    candidates.push(budget.candidate(path, verified)?);
                } else {
                    let (_, seal) = file.verify(budget)?;
                    directory.verify_entry(&name, &seal, budget)?;
                }
            }
        }
    }
    directory.seal(budget)?;
    Ok(())
}

fn limit<T>(message: &str) -> ArtifactResult<T> {
    Err(limit_error(message))
}

fn limit_error(message: &str) -> ArtifactError {
    ArtifactError::new(ArtifactErrorKind::ResourceLimit, message)
}
