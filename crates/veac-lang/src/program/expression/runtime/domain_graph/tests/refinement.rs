use super::*;

#[test]
fn content_identity_refinement_is_type_driven_and_fail_closed() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let value = content_identity(&mut graph);
    assert_eq!(domain(&value).domain_type(), DomainType::ContentIdentity);

    let before = graph.record_count();
    let error = graph
        .evaluate(DomainOperationId::Sha256.opcode(), vec![text("BAD")], 1..2)
        .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_CONTENT_IDENTITY");
    assert_eq!(graph.record_count(), before);
}

#[test]
fn non_identity_descriptions_do_not_receive_digest_refinement() {
    let (registry, budget) = standard();
    let mut graph = DomainGraphTransaction::new(&registry, &budget);
    let location = evaluate(
        &mut graph,
        DomainOperationId::ResourceFile,
        vec![text("not-a-digest")],
    );
    assert_eq!(
        domain(&location).domain_type(),
        DomainType::ResourceLocation
    );
}
