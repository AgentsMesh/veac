#[cfg(unix)]
use std::collections::BTreeSet;
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;
#[cfg(unix)]
use std::time::Instant;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, RelinkCandidate};

#[cfg(unix)]
mod budget;
mod limits;
#[cfg(unix)]
mod platform;
#[cfg(test)]
mod tests;
#[cfg(unix)]
mod walk;

#[cfg(unix)]
use budget::DiscoveryBudget;
pub use limits::*;

pub fn discover_relink_candidates(roots: &[PathBuf]) -> ArtifactResult<Vec<RelinkCandidate>> {
    discover_relink_candidates_with_limits(roots, RelinkDiscoveryLimits::default())
}

#[cfg(unix)]
pub fn discover_relink_candidates_with_limits(
    roots: &[PathBuf],
    limits: RelinkDiscoveryLimits,
) -> ArtifactResult<Vec<RelinkCandidate>> {
    discover_with_hook(roots, limits, &mut |_, _| {})
}

#[cfg(not(unix))]
pub fn discover_relink_candidates_with_limits(
    roots: &[PathBuf],
    limits: RelinkDiscoveryLimits,
) -> ArtifactResult<Vec<RelinkCandidate>> {
    limits.validate()?;
    if roots.is_empty() {
        return invalid("relink discovery requires at least one search root");
    }
    Err(ArtifactError::new(
        ArtifactErrorKind::UnsafePath,
        "descriptor-bound relink discovery is unsupported on this platform",
    ))
}

#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscoveryHookPoint {
    EntryEnumerated,
    FileHashed,
    RootVisited,
}

#[cfg(unix)]
fn discover_with_hook(
    roots: &[PathBuf],
    limits: RelinkDiscoveryLimits,
    hook: &mut impl FnMut(DiscoveryHookPoint, &Path),
) -> ArtifactResult<Vec<RelinkCandidate>> {
    let started = Instant::now();
    limits.validate()?;
    if roots.is_empty() {
        return invalid("relink discovery requires at least one search root");
    }
    if roots.len() > limits.max_entries {
        return Err(ArtifactError::new(
            ArtifactErrorKind::ResourceLimit,
            "relink discovery exceeds the search-root budget",
        ));
    }
    let mut budget = DiscoveryBudget::new(limits, started)?;
    let roots = platform::bind_roots(roots, &mut budget)?;
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    for root in roots {
        budget.observe_entry()?;
        walk::visit(
            root.directory(),
            0,
            &mut candidates,
            &mut seen,
            &mut budget,
            hook,
        )?;
        hook(DiscoveryHookPoint::RootVisited, root.path());
        budget.check_deadline()?;
        root.verify(&mut budget)?;
    }
    candidates.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(candidates)
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}
