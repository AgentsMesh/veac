use super::*;

#[test]
fn closed_constructor_remains_a_runtime_identity_operation() {
    let construct = std::hint::black_box(DomainType::new as fn(u16) -> DomainType);
    let value = construct(std::hint::black_box(DomainType::Canvas.opcode()));
    assert_eq!(value, DomainType::Canvas);
    assert_eq!(value.name(), "Canvas");
}

#[test]
fn every_closed_domain_type_exposes_its_classification_contract() {
    let values = DomainType::all().collect::<Vec<_>>();
    assert_eq!(values.len(), 214);
    assert!(values.iter().all(|value| {
        let round_trip = DomainType::from_opcode(value.opcode());
        assert_eq!(round_trip, Some(*value));
        assert_eq!(DomainType::parse(value.name()), Some(*value));
        assert_eq!(value.to_string(), value.name());
        value.is_graph_entity() || !value.is_graph_entity()
    }));
    let containers = values.iter().filter(|value| value.is_container()).count();
    let entities = values
        .iter()
        .filter(|value| value.is_graph_entity())
        .count();
    let topology = values
        .iter()
        .filter(|value| value.requires_topology_axis())
        .count();
    assert_eq!(containers, 4);
    assert_eq!(entities, 10);
    assert_eq!(topology, 37);
}

#[test]
fn invalid_closed_domain_values_fail_closed() {
    let invalid = DomainType::new(0xffff);
    assert!(std::panic::catch_unwind(|| invalid.name()).is_err());
    assert!(std::panic::catch_unwind(|| invalid.is_container()).is_err());
    assert!(std::panic::catch_unwind(|| invalid.is_graph_entity()).is_err());
    assert!(std::panic::catch_unwind(|| invalid.requires_topology_axis()).is_err());
}
