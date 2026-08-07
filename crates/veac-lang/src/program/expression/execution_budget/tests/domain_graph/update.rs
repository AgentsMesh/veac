use super::*;

#[test]
fn item_template_update_accepts_exact_bytes_and_is_atomic_one_short() {
    let registry = DomainOperationRegistry::standard();
    let measured = budget(Resource::EmittedBytes, usize::MAX);
    let mut graph = DomainGraphTransaction::new(&registry, &measured);
    let operands = prepare(&mut graph);
    evaluate(&mut graph, DomainOperationId::ItemWithTemplate, operands);
    let exact_bytes = used(&measured, Resource::EmittedBytes);

    let exact = budget(Resource::EmittedBytes, exact_bytes);
    let mut graph = DomainGraphTransaction::new(&registry, &exact);
    let operands = prepare(&mut graph);
    evaluate(&mut graph, DomainOperationId::ItemWithTemplate, operands);
    assert_eq!(used(&exact, Resource::EmittedBytes), exact_bytes);

    let short = budget(Resource::EmittedBytes, exact_bytes - 1);
    let mut graph = DomainGraphTransaction::new(&registry, &short);
    let operands = prepare(&mut graph);
    let before = snapshot(&graph, &short);
    let error =
        evaluate_result(&mut graph, DomainOperationId::ItemWithTemplate, operands).unwrap_err();
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
    let item = visual_item(graph, "item", 0);
    let contract = template(graph);
    vec![item, contract]
}
