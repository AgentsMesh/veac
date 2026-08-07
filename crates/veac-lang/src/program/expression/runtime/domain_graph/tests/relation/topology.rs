use super::*;

#[test]
fn references_across_layers_in_one_sequence_are_valid() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = visual_item(&mut graph, "first", 0);
    let second = visual_item(&mut graph, "second", 1);
    let relation = group(
        &mut graph,
        "cross-layer",
        vec![first.clone(), second.clone()],
    );
    let first_layer = layer_with(&mut graph, "first-layer", vec![first]);
    let second_layer = layer_with(&mut graph, "second-layer", vec![second]);
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
        .find(|entity| entity.domain_type() == Some(DomainType::Relation))
        .unwrap();
    let keys = relation
        .relation_references()
        .map(|entity| entity.key().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(keys, ["first", "second"]);
}

#[test]
fn reference_to_an_unattached_entity_fails_relation_topology() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let attached = visual_item(&mut graph, "attached", 0);
    let unattached = visual_item(&mut graph, "unattached", 1);
    let relation = group(&mut graph, "incomplete", vec![attached.clone(), unattached]);
    let layer = layer_with(&mut graph, "visual", vec![attached]);
    let sequence = sequence_with(&mut graph, "main", vec![layer], Some(relation));
    let entry = sequence.clone();
    let root = project_with(&mut graph, vec![sequence], entry);

    assert_freeze_error(graph, &root);
}

#[test]
fn references_across_sequences_fail_relation_topology() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let first = visual_item(&mut graph, "first", 0);
    let second = visual_item(&mut graph, "second", 1);
    let relation = group(
        &mut graph,
        "cross-sequence",
        vec![first.clone(), second.clone()],
    );
    let first_layer = layer_with(&mut graph, "first-layer", vec![first]);
    let second_layer = layer_with(&mut graph, "second-layer", vec![second]);
    let first_sequence = sequence_with(&mut graph, "first", vec![first_layer], Some(relation));
    let second_sequence = sequence_with(&mut graph, "second", vec![second_layer], None);
    let entry = first_sequence.clone();
    let root = project_with(&mut graph, vec![first_sequence, second_sequence], entry);

    assert_freeze_error(graph, &root);
}

fn assert_freeze_error(graph: DomainGraphTransaction<'_>, root: &Value) {
    let error = graph.freeze(root, 4..8).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_RELATION_TOPOLOGY");
}
