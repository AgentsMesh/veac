use super::*;

#[test]
fn const_identity_construction_and_ordering_are_runtime_exercised() {
    let value = OperationIdentity::new(DomainOperationId::Canvas, std::hint::black_box("canvas"));
    assert_eq!(value.id(), DomainOperationId::Canvas);
    assert_eq!(value.name(), "canvas");
    let values = std::hint::black_box(ordered_identities());
    assert_eq!(values.len(), COUNT);
    assert!(values
        .windows(2)
        .all(|pair| pair[0].id().opcode() < pair[1].id().opcode()));
}
