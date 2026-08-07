use super::*;

#[test]
fn relation_group_accepts_exact_bytes_and_rolls_back_one_short() {
    let registry = DomainOperationRegistry::standard();
    let measured = budget(Resource::EmittedBytes, usize::MAX);
    let mut graph = DomainGraphTransaction::new(&registry, &measured);
    let operands = prepare(&mut graph);
    evaluate(&mut graph, DomainOperationId::RelationGroup, operands);
    let exact_bytes = used(&measured, Resource::EmittedBytes);

    let exact = budget(Resource::EmittedBytes, exact_bytes);
    let mut graph = DomainGraphTransaction::new(&registry, &exact);
    let operands = prepare(&mut graph);
    evaluate(&mut graph, DomainOperationId::RelationGroup, operands);
    assert_eq!(used(&exact, Resource::EmittedBytes), exact_bytes);

    let short = budget(Resource::EmittedBytes, exact_bytes - 1);
    let mut graph = DomainGraphTransaction::new(&registry, &short);
    let operands = prepare(&mut graph);
    let before = snapshot(&graph, &short);
    let error =
        evaluate_result(&mut graph, DomainOperationId::RelationGroup, operands).unwrap_err();
    assert_atomic_failure(
        &graph,
        &short,
        error,
        before.records,
        before.bytes,
        before.entities,
    );
    assert_tainted(&mut graph);
}

fn prepare(graph: &mut DomainGraphTransaction<'_>) -> Vec<Value> {
    let first = visual_item(graph, "first", 0);
    let second = visual_item(graph, "second", 1);
    vec![
        identifier("group"),
        list(DomainType::Item, vec![first, second]),
    ]
}
