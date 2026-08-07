use super::*;

#[test]
fn foreign_forged_and_stale_relation_references_fail_closed() {
    let (registry, first_budget) = standard();
    let mut first = DomainGraphTransaction::new(&registry, &first_budget);
    let foreign = visual_item(&mut first, "foreign", 0);
    let second_budget = crate::program::expression::ExecutionBudget::default();
    let mut second = DomainGraphTransaction::new(&registry, &second_budget);
    assert_error(&mut second, vec![foreign], "DOMAIN_CROSS_GRAPH");

    let mut graph = DomainGraphTransaction::new(&registry, &first_budget);
    let canvas = canvas(&mut graph);
    let forged = Value::Domain(Arc::new(
        crate::program::expression::DomainValue::from_arena(
            graph.scope.clone(),
            domain(&canvas).arena_slot(),
            DomainType::Item,
        ),
    ));
    assert_error(&mut graph, vec![forged], "DOMAIN_HANDLE_FORGED");

    let mut graph = DomainGraphTransaction::new(&registry, &first_budget);
    let stale = visual_item(&mut graph, "stale", 0);
    update_template(&mut graph, stale.clone());
    assert_error(&mut graph, vec![stale], "DOMAIN_HANDLE_STALE");
}

#[test]
fn empty_and_duplicate_reference_sets_are_rejected_atomically() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    assert_error(&mut graph, vec![], "DOMAIN_RELATION_REFERENCES");

    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let item = visual_item(&mut graph, "same", 0);
    assert_error(
        &mut graph,
        vec![item.clone(), item],
        "DOMAIN_RELATION_REFERENCES",
    );
}

fn assert_error(graph: &mut DomainGraphTransaction<'_>, members: Vec<Value>, code: &str) {
    let before = graph.record_count();
    let error = graph
        .evaluate(
            DomainOperationId::RelationGroup.opcode(),
            vec![identifier("relation"), list(DomainType::Item, members)],
            1..2,
        )
        .unwrap_err();
    assert_eq!(error.code(), code);
    assert_eq!(graph.record_count(), before);
}
