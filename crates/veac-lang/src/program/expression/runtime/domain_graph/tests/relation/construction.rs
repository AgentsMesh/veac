use super::super::super::validation;
use super::*;

#[test]
fn relation_keeps_arbitrary_non_owning_references_across_updates() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = visual_item(&mut graph, "first", 0);
    let second = visual_item(&mut graph, "second", 1);
    let third = visual_item(&mut graph, "third", 2);
    let nodes = [&first, &second, &third]
        .map(|value| validation::node(&graph, value, DomainType::Item, &(1..2)).unwrap());
    let relation = group(
        &mut graph,
        "group",
        vec![first.clone(), second.clone(), third.clone()],
    );
    let relation_node = validation::node(&graph, &relation, DomainType::Relation, &(1..2)).unwrap();
    assert_eq!(
        graph.state.nodes[relation_node.0]
            .relation
            .as_ref()
            .unwrap()
            .as_slice(),
        nodes
    );
    assert!(graph.state.nodes[relation_node.0].children.is_empty());
    assert!(nodes
        .iter()
        .all(|node| graph.state.nodes[node.0].owner.is_none()));

    let first = update_template(&mut graph, first);
    let first_layer = layer_with(&mut graph, "first-layer", vec![first, second]);
    let second_layer = layer_with(&mut graph, "second-layer", vec![third]);
    let sequence = sequence_with(
        &mut graph,
        "main",
        vec![first_layer, second_layer],
        Some(relation),
    );
    let entry = sequence.clone();
    let root = project_with(&mut graph, vec![sequence], entry);
    let frozen = freeze(graph, &root);
    let relation = frozen
        .root_entity()
        .children()
        .next()
        .unwrap()
        .children()
        .find(|child| child.domain_type() == Some(DomainType::Relation))
        .unwrap();
    let paths = relation
        .relation_references()
        .map(|reference| reference.logical_path().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 3);
    assert_eq!(paths[0], ["project", "main", "first-layer", "first"]);
    assert_eq!(paths[2], ["project", "main", "second-layer", "third"]);
    let (first, second) = relation.relation_endpoints().unwrap();
    assert_eq!(first.key(), Some("first"));
    assert_eq!(second.key(), Some("second"));
}

#[test]
fn non_relation_entities_expose_no_relation_references() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let item = visual_item(&mut graph, "item", 0);
    let layer = layer_with(&mut graph, "layer", vec![item]);
    let sequence = sequence_with(&mut graph, "main", vec![layer], None);
    let entry = sequence.clone();
    let root = project_with(&mut graph, vec![sequence], entry);
    let frozen = freeze(graph, &root);
    assert_eq!(frozen.root_entity().relation_references().count(), 0);
    assert!(frozen.root_entity().relation_endpoints().is_none());
}
