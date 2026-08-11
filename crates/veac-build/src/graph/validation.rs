use std::collections::{BTreeMap, BTreeSet};

use crate::{
    action::ActionContract, graph::digest, BuildAction, BuildError, BuildResult, NodeId, NodeSpec,
    ValidatedGraph, MAX_GRAPH_EDGES, MAX_GRAPH_NODES, MAX_NODE_OUTPUTS,
};

pub(super) fn validate<A: BuildAction>(
    mut nodes: BTreeMap<NodeId, NodeSpec<A>>,
) -> BuildResult<ValidatedGraph<A>> {
    if nodes.is_empty() || nodes.len() > MAX_GRAPH_NODES {
        return Err(BuildError::resource_limit(format!(
            "a graph must contain between 1 and {MAX_GRAPH_NODES} nodes"
        )));
    }
    normalize(&mut nodes)?;
    validate_edges(&nodes)?;
    let topology = topology(&nodes)?;
    let actions = nodes
        .iter()
        .map(|(id, node)| Ok((id.clone(), ActionContract::capture(&node.action)?)))
        .collect::<BuildResult<BTreeMap<_, _>>>()?;
    let digest = digest::calculate(&nodes, &actions);
    Ok(ValidatedGraph {
        nodes,
        actions,
        topology,
        digest,
    })
}

fn normalize<A>(nodes: &mut BTreeMap<NodeId, NodeSpec<A>>) -> BuildResult<()> {
    let mut edge_count = 0usize;
    for node in nodes.values_mut() {
        node.resources.validate()?;
        node.outputs.sort();
        node.inputs
            .sort_by(|left, right| left.role.cmp(&right.role));
        node.order_after.sort();
        if node.outputs.is_empty() || node.outputs.len() > MAX_NODE_OUTPUTS {
            return Err(BuildError::resource_limit(format!(
                "node '{}' must declare between 1 and {MAX_NODE_OUTPUTS} outputs",
                node.id
            )));
        }
        reject_adjacent_duplicate(&node.outputs, "output", &node.id)?;
        let roles = node
            .inputs
            .iter()
            .map(|input| &input.role)
            .collect::<Vec<_>>();
        reject_adjacent_duplicate(&roles, "input role", &node.id)?;
        reject_adjacent_duplicate(&node.order_after, "order dependency", &node.id)?;
        edge_count = edge_count
            .saturating_add(node.inputs.len())
            .saturating_add(node.order_after.len());
    }
    if edge_count > MAX_GRAPH_EDGES {
        return Err(BuildError::resource_limit(
            "graph exceeds its dependency edge budget",
        ));
    }
    Ok(())
}

fn reject_adjacent_duplicate<T: Ord + std::fmt::Display>(
    values: &[T],
    subject: &str,
    id: &NodeId,
) -> BuildResult<()> {
    if let Some(value) = values
        .windows(2)
        .find(|pair| pair[0] == pair[1])
        .map(|pair| &pair[0])
    {
        Err(BuildError::invalid(format!(
            "node '{id}' declares {subject} '{value}' more than once"
        )))
    } else {
        Ok(())
    }
}

fn validate_edges<A>(nodes: &BTreeMap<NodeId, NodeSpec<A>>) -> BuildResult<()> {
    for node in nodes.values() {
        for dependency in &node.order_after {
            validate_dependency(nodes, &node.id, dependency)?;
        }
        for input in &node.inputs {
            validate_dependency(nodes, &node.id, &input.node)?;
            let producer = nodes
                .get(&input.node)
                .expect("input dependency was validated immediately above");
            if producer.outputs.binary_search(&input.output).is_err() {
                return Err(BuildError::invalid(format!(
                    "input '{}' references undeclared output '{}.{}'",
                    input.role, input.node, input.output
                )));
            }
        }
    }
    Ok(())
}

fn validate_dependency<A>(
    nodes: &BTreeMap<NodeId, NodeSpec<A>>,
    consumer: &NodeId,
    dependency: &NodeId,
) -> BuildResult<()> {
    if dependency == consumer {
        return Err(BuildError::invalid(format!(
            "node '{consumer}' cannot depend on itself"
        )));
    }
    if !nodes.contains_key(dependency) {
        return Err(BuildError::invalid(format!(
            "node '{consumer}' references missing node dependency '{dependency}'"
        )));
    }
    Ok(())
}

fn topology<A>(nodes: &BTreeMap<NodeId, NodeSpec<A>>) -> BuildResult<Vec<NodeId>> {
    let mut dependencies = BTreeMap::new();
    let mut consumers: BTreeMap<NodeId, BTreeSet<NodeId>> = BTreeMap::new();
    for (id, node) in nodes {
        let unique = node
            .inputs
            .iter()
            .map(|input| input.node.clone())
            .chain(node.order_after.iter().cloned())
            .collect::<BTreeSet<_>>();
        dependencies.insert(id.clone(), unique.len());
        for producer in unique {
            consumers.entry(producer).or_default().insert(id.clone());
        }
    }
    let mut ready = dependencies
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(id) = ready.pop_first() {
        order.push(id.clone());
        for consumer in consumers.get(&id).into_iter().flatten() {
            let count = dependencies.get_mut(consumer).expect("validated consumer");
            *count -= 1;
            if *count == 0 {
                ready.insert(consumer.clone());
            }
        }
    }
    if order.len() != nodes.len() {
        let cyclic = dependencies
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(id, _)| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(BuildError::invalid(format!(
            "graph contains a cycle involving: {cyclic}"
        )));
    }
    Ok(order)
}
