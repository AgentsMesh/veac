use super::*;

#[test]
fn const_identity_construction_and_ordering_are_runtime_exercised() {
    let value = DomainTypeIdentity::new(
        DomainType::Canvas,
        std::hint::black_box("Canvas"),
        DomainTypeClassification::TopologyValue,
    );
    assert_eq!(value.domain_type(), DomainType::Canvas);
    assert_eq!(value.name(), "Canvas");
    assert_eq!(
        value.classification(),
        DomainTypeClassification::TopologyValue
    );
    let values = std::hint::black_box(ordered_identities());
    assert_eq!(values.len(), COUNT);
    assert!(values
        .windows(2)
        .all(|pair| pair[0].domain_type().opcode() < pair[1].domain_type().opcode()));
}
