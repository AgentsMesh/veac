use super::super::validation;
use super::*;

#[test]
fn non_owning_update_is_receiver_type_driven_and_preserves_node_identity() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let item = visual_item(&mut graph, "title", 0);
    let node = validation::node(&graph, &item, DomainType::Item, &(1..2)).unwrap();
    let before = &graph.state.nodes[node.0];
    let invariant = (
        before.key.clone(),
        before.owner,
        before.children.clone(),
        before.origin_slot,
        before.entity,
    );
    let updated = update_template(&mut graph, item.clone());
    assert_eq!(
        validation::node(&graph, &updated, DomainType::Item, &(1..2)).unwrap(),
        node
    );
    let after = &graph.state.nodes[node.0];
    assert_eq!(
        (
            after.key.clone(),
            after.owner,
            after.children.clone(),
            after.origin_slot,
            after.entity,
        ),
        invariant
    );

    let contract = template(&mut graph);
    let error = graph
        .evaluate(
            DomainOperationId::ItemWithTemplate.opcode(),
            vec![item, contract],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_HANDLE_STALE");
}

#[test]
fn latest_generic_update_is_discoverable_after_freeze() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let item = visual_item(&mut graph, "title", 0);
    let item = update_template(&mut graph, item);
    let expected = template(&mut graph);
    let item = evaluate(
        &mut graph,
        DomainOperationId::ItemWithTemplate,
        vec![item, expected.clone()],
    );
    let layer = layer(&mut graph, "visual");
    let layer = attach(&mut graph, DomainOperationId::LayerWithItem, layer, item);
    let sequence = sequence(&mut graph, "main");
    let sequence = attach(
        &mut graph,
        DomainOperationId::SequenceWithLayer,
        sequence,
        layer,
    );
    let entry = sequence.clone();
    let project = project(&mut graph, "demo");
    let project = attach(
        &mut graph,
        DomainOperationId::ProjectWithSequence,
        project,
        sequence,
    );
    let project = evaluate(
        &mut graph,
        DomainOperationId::ProjectEntry,
        vec![project, entry],
    );
    let frozen = freeze(graph, &project);
    let item = frozen
        .root_entity()
        .children()
        .next()
        .unwrap()
        .children()
        .next()
        .unwrap()
        .children()
        .next()
        .unwrap();
    let update = item
        .latest_update(DomainOperationId::ItemWithTemplate)
        .unwrap();
    assert_eq!(update.get(1), Some(&expected));
}

#[test]
fn attached_and_cross_graph_updates_fail_before_mutation() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let item = visual_item(&mut graph, "title", 0);
    let retained = item.clone();
    let layer = layer(&mut graph, "visual");
    attach(&mut graph, DomainOperationId::LayerWithItem, layer, item);
    let contract = template(&mut graph);
    assert_update_error(&mut graph, retained, contract, "DOMAIN_TOPOLOGY_CLOSED");

    let first_budget = crate::program::expression::ExecutionBudget::default();
    let mut first = DomainGraphTransaction::new(&registry, &first_budget);
    let foreign = template(&mut first);
    let second_budget = crate::program::expression::ExecutionBudget::default();
    let mut second = DomainGraphTransaction::new(&registry, &second_budget);
    let item = visual_item(&mut second, "title", 0);
    assert_update_error(&mut second, item, foreign, "DOMAIN_CROSS_GRAPH");
}

fn assert_update_error(
    graph: &mut DomainGraphTransaction<'_>,
    item: Value,
    contract: Value,
    code: &str,
) {
    let before = graph.record_count();
    let error = graph
        .evaluate(
            DomainOperationId::ItemWithTemplate.opcode(),
            vec![item, contract],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), code);
    assert_eq!(graph.record_count(), before);
}
