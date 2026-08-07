use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{ExecutableManifest, ProjectId, SequenceId, TemporalProgramLibrary};

use super::{ResolvedInput, ResolvedOutput, ResolvedSequence};

pub const RENDER_PLAN_SCHEMA_ID: &str = "https://veac.dev/schemas/render-plan";
pub const CURRENT_RENDER_PLAN_VERSION: u32 = 6;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedRenderPlan {
    pub header: RenderPlanHeader,
    pub output: ResolvedOutput,
    /// Inputs sorted by `PlanInputId`; machine-local paths never enter the render plan.
    pub inputs: Vec<ResolvedInput>,
    /// Reachable temporal closure, sorted by stable typed IDs.
    pub temporal: TemporalProgramLibrary,
    /// Reachable sequences in dependency-first order, with IDs as the stable graph tie-breaker.
    pub sequences: Vec<ResolvedSequence>,
    pub entry_sequence_id: SequenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderPlanHeader {
    pub schema: String,
    pub schema_version: u32,
    pub source: PlanSource,
    pub resolver: ResolverFingerprint,
    pub cache: PlanCacheIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanSource {
    pub project_id: ProjectId,
    #[schemars(range(max = 9007199254740991u64))]
    pub revision: u64,
    pub timebase: u32,
    pub semantic_hash: String,
    pub snapshot_hash: String,
    pub executable: ExecutableManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolverFingerprint {
    pub resolver_version: String,
    pub stream_selection_policy: String,
    pub effect_registry_version: String,
    pub capability_profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanCacheIdentity {
    pub project_semantic_sha256: String,
    pub project_snapshot_sha256: String,
    pub executable_manifest_sha256: String,
    pub temporal_library_sha256: String,
    pub resolver_sha256: String,
}
