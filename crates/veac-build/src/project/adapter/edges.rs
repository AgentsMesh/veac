use std::collections::BTreeMap;

use veac_artifact::ContentDigest;
use veac_project::{ProjectOutput, ResolvedTargetEdge, TargetInstanceId};

use super::ProjectArtifact;
use crate::{BuildError, BuildResult, InputSlot, NodeId, NodeSpec, OutputSlot, ProjectAction};

pub(super) fn bind(
    edge: &ResolvedTargetEdge,
    ids: &BTreeMap<TargetInstanceId, NodeId>,
    specs: &mut BTreeMap<TargetInstanceId, NodeSpec<ProjectAction>>,
) -> BuildResult<()> {
    let dependency = ids
        .get(&edge.dependency)
        .ok_or_else(|| BuildError::invalid("project edge dependency is missing"))?;
    let mut consumer = specs
        .remove(&edge.consumer)
        .ok_or_else(|| BuildError::invalid("project edge consumer is missing"))?;
    consumer = if let Some(output) = &edge.output {
        let producer = specs
            .get(&edge.dependency)
            .ok_or_else(|| BuildError::invalid("project edge producer is missing"))?;
        let output = producer.output_ref(&OutputSlot::<ProjectArtifact>::new(output.as_str())?);
        consumer.bind(&InputSlot::new(edge_role(edge))?, &output)
    } else {
        consumer.after(dependency)
    };
    specs.insert(edge.consumer.clone(), consumer);
    Ok(())
}

fn edge_role(edge: &ResolvedTargetEdge) -> String {
    let value = format!(
        "{}|{}|{}|{}",
        edge.binding.as_ref().map_or("", |value| value.as_str()),
        edge.dependency,
        edge.output.as_ref().map_or("", |value| value.as_str()),
        edge.consumer
    );
    format!("edge-{}", &ContentDigest::sha256(value).value[..32])
}

pub(super) fn output_slot(output: &ProjectOutput) -> BuildResult<OutputSlot<ProjectArtifact>> {
    OutputSlot::new(output.id().as_str())
}

pub(super) fn instance_node_id(instance: &TargetInstanceId) -> BuildResult<NodeId> {
    let digest = ContentDigest::sha256(instance.as_str());
    NodeId::new(format!("instance-{}", &digest.value[..32]))
}
