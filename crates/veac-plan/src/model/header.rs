use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{ProjectId, SequenceId};

use super::{ResolvedInput, ResolvedOutput, ResolvedSequence};

pub const RENDER_PLAN_SCHEMA_ID: &str = "https://veac.dev/schemas/render-plan";
pub const CURRENT_RENDER_PLAN_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedRenderPlan {
    pub header: RenderPlanHeader,
    pub output: ResolvedOutput,
    /// Inputs sorted by `PlanInputId`; machine-local paths never enter the render plan.
    pub inputs: Vec<ResolvedInput>,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolverFingerprint {
    pub resolver_version: String,
    pub stream_selection_policy: String,
    pub effect_registry_version: String,
    pub capability_profile: String,
}
