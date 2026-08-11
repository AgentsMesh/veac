use std::collections::BTreeMap;

use veac_artifact::ContentDigest;

use crate::{
    action::ActionContract, stable::StableBytes, NodeId, NodeSpec, BUILD_GRAPH_CONTRACT_VERSION,
};

pub(super) fn calculate<A>(
    nodes: &BTreeMap<NodeId, NodeSpec<A>>,
    actions: &BTreeMap<NodeId, ActionContract>,
) -> ContentDigest {
    let mut bytes = StableBytes::default();
    bytes.text("veac-artifact-graph");
    bytes.u32(BUILD_GRAPH_CONTRACT_VERSION);
    bytes.u32(nodes.len() as u32);
    for (id, node) in nodes {
        let action = &actions[id];
        bytes.text(id.as_str());
        bytes.text(&action.kind);
        bytes.u32(action.version);
        bytes.bytes(&action.bytes);
        bytes.u16(node.resources.cpu_units);
        bytes.u32(node.resources.memory_mb);
        bytes.u16(node.resources.gpu_units);
        bytes.u32(node.outputs.len() as u32);
        for output in &node.outputs {
            bytes.text(output.as_str());
        }
        bytes.u32(node.inputs.len() as u32);
        for input in &node.inputs {
            bytes.text(input.role.as_str());
            bytes.text(input.node.as_str());
            bytes.text(input.output.as_str());
        }
        bytes.u32(node.order_after.len() as u32);
        for dependency in &node.order_after {
            bytes.text(dependency.as_str());
        }
    }
    bytes.digest()
}
