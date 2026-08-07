use super::*;

#[test]
fn closed_constructor_remains_a_runtime_identity_operation() {
    let construct = std::hint::black_box(DomainType::new as fn(u16) -> DomainType);
    let value = construct(std::hint::black_box(DomainType::Canvas.opcode()));
    assert_eq!(value, DomainType::Canvas);
    assert_eq!(value.name(), "Canvas");
}
