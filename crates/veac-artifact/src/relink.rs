use std::path::PathBuf;

use veac_ir::MediaIdentity;
use veac_plan::{PlanInputId, ResolvedRenderPlan};

use crate::{ArtifactResult, ExecutionBindings};

mod discover;
pub use discover::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelinkCandidate {
    pub path: PathBuf,
    pub identity: MediaIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousRelink {
    pub input_id: PlanInputId,
    pub candidates: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelinkResolution {
    pub bindings: ExecutionBindings,
    pub unresolved: Vec<PlanInputId>,
    pub ambiguous: Vec<AmbiguousRelink>,
}

pub fn resolve_relinks(
    plan: &ResolvedRenderPlan,
    candidates: &[RelinkCandidate],
) -> ArtifactResult<RelinkResolution> {
    let mut bindings = ExecutionBindings::default();
    let mut unresolved = Vec::new();
    let mut ambiguous = Vec::new();
    for input in &plan.inputs {
        let mut matches: Vec<_> = candidates
            .iter()
            .filter(|candidate| candidate.identity == input.observed_identity)
            .map(|candidate| candidate.path.clone())
            .collect();
        matches.sort();
        matches.dedup();
        match matches.as_slice() {
            [] => unresolved.push(input.id.clone()),
            [path] => {
                bindings.bind_original(input, path.clone())?;
            }
            _ => ambiguous.push(AmbiguousRelink {
                input_id: input.id.clone(),
                candidates: matches,
            }),
        }
    }
    Ok(RelinkResolution {
        bindings,
        unresolved,
        ambiguous,
    })
}
