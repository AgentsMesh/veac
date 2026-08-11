use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ContentDigest;

use crate::{ArtifactOutputs, NodeCacheKey, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Executed,
    CacheHit,
    Failed,
    Blocked,
    Cancelled,
}

impl NodeStatus {
    pub fn is_success(self) -> bool {
        matches!(self, Self::Executed | Self::CacheHit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NodeReceipt {
    pub node_id: NodeId,
    pub status: NodeStatus,
    pub cache_key: Option<NodeCacheKey>,
    pub outputs: Option<ArtifactOutputs>,
    pub message: Option<String>,
}

impl NodeReceipt {
    pub(crate) fn new(
        node_id: NodeId,
        status: NodeStatus,
        cache_key: Option<NodeCacheKey>,
        outputs: Option<ArtifactOutputs>,
        message: Option<String>,
    ) -> Self {
        Self {
            node_id,
            status,
            cache_key,
            outputs,
            message,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildOutcome {
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildReceipt {
    pub graph_digest: ContentDigest,
    pub outcome: BuildOutcome,
    pub nodes: Vec<NodeReceipt>,
}

pub fn build_receipt_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schemars::schema_for!(BuildReceipt))
}

impl BuildReceipt {
    pub fn node(&self, id: &NodeId) -> Option<&NodeReceipt> {
        self.nodes.iter().find(|receipt| receipt.node_id == *id)
    }
}
