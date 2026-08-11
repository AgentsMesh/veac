use std::collections::BTreeMap;

use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_project::{DeliveryKind, ProjectOutput, ResolvedTargetGraph, TargetInstanceId};

use super::PlannedDelivery;
use crate::{BuildError, BuildResult, NodeId};

pub(super) fn deliveries(
    graph: &ResolvedTargetGraph,
    ids: &BTreeMap<TargetInstanceId, NodeId>,
) -> BuildResult<Vec<PlannedDelivery>> {
    let mut values = Vec::new();
    for instance in &graph.instances {
        for delivery in &instance.deliveries {
            let output = instance
                .outputs
                .iter()
                .find(|output| output.id() == &delivery.output)
                .ok_or_else(|| BuildError::invalid("project delivery output is missing"))?;
            let directory = matches!(output, ProjectOutput::Directory { .. });
            if directory != (delivery.kind == DeliveryKind::Directory) {
                return Err(BuildError::invalid(
                    "directory delivery kind must match a directory project output",
                ));
            }
            values.push(PlannedDelivery {
                node: ids[&instance.id].clone(),
                instance: instance.id.clone(),
                delivery: delivery.clone(),
            });
        }
    }
    Ok(values)
}

pub(super) fn parse_digest(value: &str) -> BuildResult<ContentDigest> {
    let value = value
        .strip_prefix("sha256:")
        .ok_or_else(|| BuildError::invalid("manifest digest must use the sha256 scheme"))?;
    let digest = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: value.to_owned(),
    };
    digest
        .validate()
        .map_err(|error| BuildError::invalid(format!("invalid manifest digest: {error}")))?;
    Ok(digest)
}
