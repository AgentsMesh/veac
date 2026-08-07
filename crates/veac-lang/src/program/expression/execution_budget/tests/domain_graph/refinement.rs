use super::*;

#[test]
fn invalid_content_identity_consumes_no_graph_budget_before_tainting() {
    let registry = DomainOperationRegistry::standard();
    let execution = budget(Resource::EmittedBytes, 0);
    let mut graph = DomainGraphTransaction::new(&registry, &execution);
    let error =
        evaluate_result(&mut graph, DomainOperationId::Sha256, vec![text("BAD")]).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_CONTENT_IDENTITY");
    assert_eq!(graph.record_count(), 0);
    assert_eq!(used(&execution, Resource::EmittedBytes), 0);
    assert_eq!(used(&execution, Resource::EmittedEntities), 0);
    assert_tainted(&mut graph);
}
