use super::*;

#[test]
fn image_resource_accepts_exact_limits_and_rolls_back_one_short() {
    let registry = DomainOperationRegistry::standard();
    let measured = ExecutionBudget::with_resource_limits(unlimited());
    let mut graph = DomainGraphTransaction::new(&registry, &measured);
    let operands = prepare(&mut graph);
    evaluate(&mut graph, DomainOperationId::ImageResource, operands);
    let exact_bytes = used(&measured, Resource::EmittedBytes);

    let mut exact_limits = unlimited();
    set_limit(&mut exact_limits, Resource::EmittedEntities, 1);
    set_limit(&mut exact_limits, Resource::EmittedBytes, exact_bytes);
    let exact = ExecutionBudget::with_resource_limits(exact_limits);
    let mut graph = DomainGraphTransaction::new(&registry, &exact);
    let operands = prepare(&mut graph);
    evaluate(&mut graph, DomainOperationId::ImageResource, operands);
    assert_eq!(used(&exact, Resource::EmittedEntities), 1);
    assert_eq!(used(&exact, Resource::EmittedBytes), exact_bytes);

    for (resource, limit) in [
        (Resource::EmittedEntities, 0),
        (Resource::EmittedBytes, exact_bytes - 1),
    ] {
        let short = budget(resource, limit);
        let mut graph = DomainGraphTransaction::new(&registry, &short);
        let operands = prepare(&mut graph);
        let before = snapshot(&graph, &short);
        let error =
            evaluate_result(&mut graph, DomainOperationId::ImageResource, operands).unwrap_err();
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
}

fn prepare(graph: &mut DomainGraphTransaction<'_>) -> Vec<Value> {
    let location = evaluate(graph, DomainOperationId::ResourceFile, vec![text("a.png")]);
    let identity = evaluate(graph, DomainOperationId::Sha256, vec![text(DIGEST)]);
    vec![identifier("image"), location, identity]
}
