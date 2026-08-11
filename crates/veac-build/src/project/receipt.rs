use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ContentDigest;
use veac_project::{DeliveryId, TargetId, TargetInstanceId};

use crate::{
    BuildAction, BuildError, BuildOutcome, BuildReceipt, BuildResult, NodeStatus, ProjectBuildPlan,
};

pub const PROJECT_BUILD_RECEIPT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectBuildReceipt {
    pub version: u32,
    pub manifest_digest: ContentDigest,
    pub graph_digest: ContentDigest,
    pub outcome: ProjectBuildOutcome,
    pub nodes: Vec<ProjectNodeProvenance>,
    pub deliveries: Vec<ProjectDeliveryReceipt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectBuildOutcome {
    Succeeded,
    BuildFailed,
    Cancelled,
    DeliveryFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectNodeProvenance {
    pub node_id: String,
    pub instance: TargetInstanceId,
    pub target: TargetId,
    pub action_kind: String,
    pub action_digest: ContentDigest,
    pub status: ProjectNodeStatus,
    pub cache_key: Option<ContentDigest>,
    pub outputs: Vec<ProjectArtifactProvenance>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectNodeStatus {
    Executed,
    CacheHit,
    Failed,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectArtifactProvenance {
    pub output: String,
    pub artifact: ContentDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectDeliveryReceipt {
    pub instance: TargetInstanceId,
    pub delivery: DeliveryId,
    pub destination: String,
    pub artifact: Option<ContentDigest>,
    pub status: DeliveryStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Published,
    Failed,
    Skipped,
}

impl ProjectBuildReceipt {
    pub(super) fn capture(plan: &ProjectBuildPlan, build: &BuildReceipt) -> BuildResult<Self> {
        let mut nodes = Vec::with_capacity(build.nodes.len());
        for receipt in &build.nodes {
            let identity = plan.nodes.get(&receipt.node_id).ok_or_else(|| {
                BuildError::invalid("build receipt contains an unknown project node")
            })?;
            let action = plan
                .graph
                .node(&receipt.node_id)
                .expect("project plan node")
                .action();
            let action_digest = ContentDigest::sha256(action.canonical_bytes()?);
            let outputs = receipt
                .outputs
                .iter()
                .flat_map(|outputs| outputs.iter())
                .map(|(output, artifact)| ProjectArtifactProvenance {
                    output: output.as_str().to_owned(),
                    artifact: artifact.clone(),
                })
                .collect();
            nodes.push(ProjectNodeProvenance {
                node_id: receipt.node_id.to_string(),
                instance: identity.instance.clone(),
                target: identity.target.clone(),
                action_kind: action.kind_name().to_owned(),
                action_digest,
                status: node_status(receipt.status),
                cache_key: receipt.cache_key.as_ref().map(|key| key.digest().clone()),
                outputs,
                message: receipt.message.clone(),
            });
        }
        let outcome = match build.outcome {
            BuildOutcome::Succeeded => ProjectBuildOutcome::Succeeded,
            BuildOutcome::Failed => ProjectBuildOutcome::BuildFailed,
            BuildOutcome::Cancelled => ProjectBuildOutcome::Cancelled,
        };
        Ok(Self {
            version: PROJECT_BUILD_RECEIPT_VERSION,
            manifest_digest: plan.manifest_digest.clone(),
            graph_digest: build.graph_digest.clone(),
            outcome,
            nodes,
            deliveries: Vec::new(),
        })
    }
}

pub fn canonical_project_receipt_bytes(value: &ProjectBuildReceipt) -> BuildResult<Vec<u8>> {
    match serde_json_canonicalizer::to_vec(value) {
        Ok(bytes) => Ok(bytes),
        Err(error) => Err(BuildError::invalid(format!(
            "project receipt cannot be canonicalized: {error}"
        ))),
    }
}

pub fn project_build_receipt_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schemars::schema_for!(ProjectBuildReceipt))
}

fn node_status(status: NodeStatus) -> ProjectNodeStatus {
    match status {
        NodeStatus::Executed => ProjectNodeStatus::Executed,
        NodeStatus::CacheHit => ProjectNodeStatus::CacheHit,
        NodeStatus::Failed => ProjectNodeStatus::Failed,
        NodeStatus::Blocked => ProjectNodeStatus::Blocked,
        NodeStatus::Cancelled => ProjectNodeStatus::Cancelled,
    }
}
