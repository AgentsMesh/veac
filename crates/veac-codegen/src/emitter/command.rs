use std::path::PathBuf;

use veac_artifact::ContentDigest;
use veac_plan::canonical::MediaIdentity;

mod arguments;
mod filter;
mod requirement;
mod task;

pub use filter::{
    BackendFilterBinding, BackendFilterContract, BackendFilterEscape, BackendInternalAccess,
};
pub use requirement::{BackendCapabilityKind, BackendRequirement};
pub use task::*;

/// An executable backend bundle issued only by [`super::emit_all`].
///
/// Bundle internals are intentionally read-only outside codegen. This keeps callers from turning
/// arbitrary FFmpeg arguments into an executable bundle while still allowing inspection, logging,
/// and runtime validation.
///
/// ```compile_fail
/// use veac_artifact::ContentDigest;
/// use veac_codegen::emitter::BackendBundle;
///
/// let bundle = BackendBundle {
///     plan_identity: ContentDigest::sha256(b"forged-plan"),
///     substitution_proof: ContentDigest::sha256(b"forged-substitution"),
///     protected_resources: vec![],
///     requirements: vec![],
///     tasks: vec![],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendBundle {
    plan_identity: ContentDigest,
    substitution_proof: ContentDigest,
    protected_resources: Vec<BackendResource>,
    requirements: Vec<BackendRequirement>,
    tasks: Vec<BackendTask>,
}

impl BackendBundle {
    pub(super) fn new(
        plan_identity: ContentDigest,
        substitution_proof: ContentDigest,
        protected_resources: Vec<BackendResource>,
        requirements: Vec<BackendRequirement>,
        tasks: Vec<BackendTask>,
    ) -> Self {
        Self {
            plan_identity,
            substitution_proof,
            protected_resources,
            requirements,
            tasks,
        }
    }

    pub fn plan_identity(&self) -> &ContentDigest {
        &self.plan_identity
    }

    pub fn substitution_proof(&self) -> &ContentDigest {
        &self.substitution_proof
    }

    pub fn protected_resources(&self) -> &[BackendResource] {
        &self.protected_resources
    }

    pub fn requirements(&self) -> &[BackendRequirement] {
        &self.requirements
    }

    pub fn tasks(&self) -> &[BackendTask] {
        &self.tasks
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendResource {
    pub path: PathBuf,
    pub expected_identity: MediaIdentity,
}
