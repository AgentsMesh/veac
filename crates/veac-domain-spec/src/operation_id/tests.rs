use super::*;

#[test]
fn closed_constructor_remains_a_runtime_identity_operation() {
    let construct = std::hint::black_box(DomainOperationId::new as fn(u16) -> DomainOperationId);
    let value = construct(std::hint::black_box(DomainOperationId::Canvas.opcode()));
    assert_eq!(value, DomainOperationId::Canvas);
    assert_eq!(value.name(), "canvas");
}
