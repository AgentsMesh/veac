use veac_artifact::ContentDigest;
use veac_codegen::emitter::{BackendBundle, BackendRequirement, BackendResource, BackendTask};

/// Runtime-owned copy used for validation, staging, and malformed-contract unit tests.
///
/// Production callers can only reach this representation through a sealed codegen bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::executor) struct RuntimeBundle {
    pub plan_identity: ContentDigest,
    pub substitution_proof: ContentDigest,
    pub protected_resources: Vec<BackendResource>,
    pub requirements: Vec<BackendRequirement>,
    pub tasks: Vec<BackendTask>,
}

impl RuntimeBundle {
    pub(in crate::executor) fn from_backend(bundle: &BackendBundle) -> Self {
        Self {
            plan_identity: bundle.plan_identity().clone(),
            substitution_proof: bundle.substitution_proof().clone(),
            protected_resources: bundle.protected_resources().to_vec(),
            requirements: bundle.requirements().to_vec(),
            tasks: bundle.tasks().to_vec(),
        }
    }
}
