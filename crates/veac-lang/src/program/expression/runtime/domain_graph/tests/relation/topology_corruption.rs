use super::super::super::state::{NodeId, RelationReferences};
use super::super::super::validation;
use super::*;

#[test]
fn corrupt_relation_owner_and_children_fail_closed() {
    corrupt(|graph, nodes| graph.state.nodes[nodes.relation.0].owner = None);
    corrupt(|graph, nodes| graph.state.nodes[nodes.relation.0].owner = Some(nodes.layer));
    corrupt(|graph, nodes| {
        graph.state.nodes[nodes.relation.0]
            .children
            .push(nodes.first)
    });
}

#[test]
fn corrupt_reference_storage_fails_closed() {
    corrupt(|graph, nodes| graph.state.nodes[nodes.relation.0].relation = None);
    corrupt(|graph, nodes| {
        graph.state.nodes[nodes.relation.0].relation = Some(RelationReferences::new(vec![]))
    });
    corrupt(|graph, nodes| {
        graph.state.nodes[nodes.relation.0].relation =
            Some(RelationReferences::new(vec![nodes.first, nodes.first]))
    });
    corrupt(|graph, nodes| {
        graph.state.nodes[nodes.relation.0].relation =
            Some(RelationReferences::new(vec![NodeId(usize::MAX)]))
    });
}

#[test]
fn corrupt_referenced_entities_fail_closed() {
    corrupt(|graph, nodes| graph.state.nodes[nodes.first.0].entity = false);
    corrupt(|graph, nodes| graph.state.nodes[nodes.first.0].owner = None);
    corrupt(|graph, nodes| graph.state.nodes[nodes.first.0].owner = Some(nodes.relation));
}

#[test]
fn corrupt_ownership_cycles_terminate_without_panicking() {
    corrupt(|graph, nodes| {
        graph.state.nodes[nodes.first.0].owner = Some(nodes.layer);
        graph.state.nodes[nodes.layer.0].owner = Some(nodes.first);
    });
    corrupt(|graph, nodes| graph.state.nodes[nodes.first.0].owner = Some(nodes.first));
}

#[derive(Clone, Copy)]
struct Nodes {
    relation: NodeId,
    first: NodeId,
    layer: NodeId,
}

fn corrupt(mutation: impl FnOnce(&mut DomainGraphTransaction<'_>, Nodes)) {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = visual_item(&mut graph, "first", 0);
    let second = visual_item(&mut graph, "second", 1);
    let first_node = node(&graph, &first, DomainType::Item);
    let relation = group(&mut graph, "group", vec![first.clone(), second.clone()]);
    let relation_node = node(&graph, &relation, DomainType::Relation);
    let layer = layer_with(&mut graph, "visual", vec![first, second]);
    let layer_node = node(&graph, &layer, DomainType::Layer);
    let sequence = sequence_with(&mut graph, "main", vec![layer], Some(relation));
    let entry = sequence.clone();
    let root = project_with(&mut graph, vec![sequence], entry);
    mutation(
        &mut graph,
        Nodes {
            relation: relation_node,
            first: first_node,
            layer: layer_node,
        },
    );
    let error = graph.freeze(&root, 1..2).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_RELATION_TOPOLOGY");
}

fn node(graph: &DomainGraphTransaction<'_>, value: &Value, kind: DomainType) -> NodeId {
    validation::node(graph, value, kind, &(1..2)).unwrap()
}
