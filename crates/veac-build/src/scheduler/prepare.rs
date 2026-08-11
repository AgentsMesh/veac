use std::collections::BTreeMap;

use crate::{
    stable::StableBytes, BuildAction, BuildError, BuildResult, NodeCacheKey, NodeExecutor, NodeId,
    NodeReceipt, ResolvedInput, ValidatedGraph, BUILD_GRAPH_CONTRACT_VERSION,
};

#[derive(Debug, Clone)]
pub(super) struct PreparedNode {
    pub inputs: Vec<ResolvedInput>,
    pub key: NodeCacheKey,
}

pub(super) fn prepare<A: BuildAction>(
    graph: &ValidatedGraph<A>,
    executor: &dyn NodeExecutor<A>,
    id: &NodeId,
    receipts: &BTreeMap<NodeId, NodeReceipt>,
) -> BuildResult<PreparedNode> {
    let node = graph.node(id).expect("validated node");
    let implementation = executor.implementation_identity(node.action())?;
    implementation.validate().map_err(|error| {
        BuildError::invalid(format!(
            "node executor implementation identity is invalid: {error}"
        ))
    })?;
    let mut inputs = Vec::with_capacity(node.inputs.len());
    for binding in &node.inputs {
        let producer = receipts
            .get(&binding.node)
            .expect("ready node has a producer receipt");
        let digest = producer
            .outputs
            .as_ref()
            .expect("successful producer has outputs")
            .get(&binding.output)
            .expect("validated producer has its declared output");
        inputs.push(ResolvedInput {
            role: binding.role.clone(),
            producer: binding.node.clone(),
            output: binding.output.clone(),
            digest: digest.clone(),
        });
    }
    let contract = graph.action_contract(id);
    let mut stable = StableBytes::default();
    stable.text("veac-artifact-node");
    stable.u32(BUILD_GRAPH_CONTRACT_VERSION);
    stable.text(&contract.kind);
    stable.u32(contract.version);
    stable.text(&implementation.value);
    stable.bytes(&contract.bytes);
    stable.u32(node.outputs.len() as u32);
    for output in &node.outputs {
        stable.text(output.as_str());
    }
    stable.u32(inputs.len() as u32);
    for input in &inputs {
        stable.text(input.role.as_str());
        stable.text(&input.digest.value);
    }
    let key = NodeCacheKey::computation(&contract.kind, contract.version, stable.digest())?;
    Ok(PreparedNode { inputs, key })
}

pub(super) fn validate_outputs<A>(
    graph: &ValidatedGraph<A>,
    id: &NodeId,
    outputs: &crate::ArtifactOutputs,
) -> BuildResult<()> {
    let node = graph.node(id).expect("validated node");
    let actual = outputs.iter().map(|(name, _)| name).collect::<Vec<_>>();
    let expected = node.outputs.iter().collect::<Vec<_>>();
    if actual != expected {
        return Err(BuildError::invalid(format!(
            "node '{id}' produced outputs that do not match its declaration"
        )));
    }
    Ok(())
}
