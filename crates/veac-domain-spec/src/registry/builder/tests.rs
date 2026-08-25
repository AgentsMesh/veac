use super::*;

#[test]
fn retained_byte_accumulation_overflow_fails_closed() {
    let mut builder = Builder::new(
        DomainOpsetVersion::CURRENT,
        super::super::super::MAX_DOMAIN_OPERATIONS,
        usize::MAX,
    );
    builder.retained_bytes = usize::MAX;
    let contract = crate::catalog::contract(DomainOperationId::Canvas);
    let error = builder.insert(Arc::new(contract)).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_REGISTRY_LIMIT");
    assert!(error.message().contains("retained"));
}
