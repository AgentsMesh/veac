use super::*;

#[test]
fn insertion_order_cannot_change_contract_order_or_digest() {
    let first = DomainOperationRegistry::standard();
    let mut reversed = all_contracts();
    reversed.reverse();
    let second = rebuild(reversed, MAX_DOMAIN_OPERATIONS, MAX_DOMAIN_REGISTRY_BYTES).unwrap();
    assert_eq!(first.digest(), second.digest());
    let first_ids = first
        .contracts()
        .map(|value| value.id())
        .collect::<Vec<_>>();
    let second_ids = second
        .contracts()
        .map(|value| value.id())
        .collect::<Vec<_>>();
    assert_eq!(first_ids, second_ids);
}

#[test]
fn unsupported_and_incomplete_opsets_fail_closed() {
    let error = DomainOperationRegistry::for_version(DomainOpsetVersion::V4).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPSET_UNSUPPORTED");
    assert!(error.message().contains("version 4"));
    assert_eq!(error.to_string(), error.message());
    let current = DomainOperationRegistry::for_version(DomainOpsetVersion::V8).unwrap();
    assert_eq!(current.version(), DomainOpsetVersion::CURRENT);

    let mut incomplete = all_contracts();
    incomplete.pop();
    let error = rebuild(incomplete, MAX_DOMAIN_OPERATIONS, MAX_DOMAIN_REGISTRY_BYTES).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_MISSING");
}

#[test]
fn duplicate_ids_and_names_are_rejected_deterministically() {
    let canvas = builtin(DomainOperationId::Canvas);
    let error = rebuild(
        vec![canvas.clone(), canvas],
        MAX_DOMAIN_OPERATIONS,
        MAX_DOMAIN_REGISTRY_BYTES,
    )
    .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_DUPLICATE_ID");

    let duplicate_name = operation(
        DomainOperationId::FrameRate,
        DomainOperationId::Canvas.name(),
        Vec::new(),
        domain(DomainType::FrameRate),
        DomainInstructionKind::DomainConstruct,
        Effect::Pure,
    );
    let error = rebuild(
        vec![builtin(DomainOperationId::Canvas), duplicate_name],
        MAX_DOMAIN_OPERATIONS,
        MAX_DOMAIN_REGISTRY_BYTES,
    )
    .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_DUPLICATE_NAME");
}

#[test]
fn count_and_retained_limits_are_atomic_and_exact() {
    let values = all_contracts();
    let exact = DomainOperationRegistry::standard().retained_bytes();
    let accepted = rebuild(values.clone(), DomainOperationId::all().len(), exact).unwrap();
    assert_eq!(accepted.retained_bytes(), exact);

    let error = rebuild(values.clone(), DomainOperationId::all().len(), exact - 1).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_REGISTRY_LIMIT");
    let error = rebuild(
        values,
        DomainOperationId::all().len() - 1,
        MAX_DOMAIN_REGISTRY_BYTES,
    )
    .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_REGISTRY_LIMIT");
}

#[test]
fn registry_digest_is_byte_pinned() {
    let digest = DomainOperationRegistry::standard().digest().to_string();
    assert_eq!(
        digest,
        "f02bebce0d1b43cc29e6a5aa260e869ede2a8abfeb75f8e6dee0af16859936cb"
    );
}
