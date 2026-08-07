use std::sync::Arc;

use super::super::domain::{DomainGraphScope, DomainValue, LOGICAL_DOMAIN_HANDLE_BYTES};
use super::super::{ListValue, MapValue, MapValueEntry, TupleValue, Value};
use crate::program::expression::{MapKeyType, PrimitiveType, ValueType};
use crate::program::{
    DomainType, FieldDefinition, FieldIndex, StructDefinition, TypeDefinition, TypeDefinitionKind,
    TypeRegistry, TypeRegistryBuilder,
};

fn handle(graph: &DomainGraphScope, slot: u32, value_type: DomainType) -> Value {
    Value::Domain(Arc::new(DomainValue::from_arena(
        graph.clone(),
        slot,
        value_type,
    )))
}

fn graph() -> DomainGraphScope {
    DomainGraphScope::fresh()
}

fn pair_registry() -> (TypeRegistry, crate::program::TypeId) {
    let definition = TypeDefinition::new(
        "domain-test.veac",
        "Pair",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![
            FieldDefinition::new(
                FieldIndex::new(0),
                "left",
                ValueType::domain(DomainType::Project),
            ),
            FieldDefinition::new(
                FieldIndex::new(1),
                "right",
                ValueType::domain(DomainType::Project),
            ),
        ])),
    );
    let id = definition.type_ref().id();
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(definition)).unwrap();
    (builder.finish().unwrap(), id)
}

#[test]
fn opaque_handles_expose_semantics_without_exposing_storage() {
    let graph = graph();
    let project = handle(&graph, 7, DomainType::Project);
    let clone = project.clone();
    let Value::Domain(value) = &project else {
        unreachable!()
    };
    assert_eq!(value.domain_type(), DomainType::Project);
    assert!(value.is_container());
    assert_eq!(value.arena_slot(), 7);
    assert_eq!(project, clone);
    assert_eq!(project.render(), "<Project>");
    assert_eq!(project.retained_bytes(), LOGICAL_DOMAIN_HANDLE_BYTES);
    assert_eq!(project.evaluated_bytes(), LOGICAL_DOMAIN_HANDLE_BYTES);
    assert!(format!("{project:?}").contains("<opaque>"));

    let style = handle(&graph, 8, DomainType::TextStyle);
    let Value::Domain(style) = style else {
        unreachable!()
    };
    assert!(!style.is_container());
}

#[test]
fn aggregate_construction_rejects_cross_graph_handles_atomically() {
    let first = graph();
    let second = graph();
    let left = handle(&first, 0, DomainType::Item);
    let right = handle(&second, 0, DomainType::Item);
    let error = ListValue::new(
        ValueType::domain(DomainType::Item),
        vec![left.clone(), right.clone()],
    )
    .unwrap_err();
    assert_eq!(error.code(), "VALUE_DOMAIN_CROSS_GRAPH");

    let valid = handle(&first, 1, DomainType::Item);
    ListValue::new(
        ValueType::domain(DomainType::Item),
        vec![left.clone(), valid],
    )
    .unwrap();
    assert_eq!(
        TupleValue::new(vec![left.clone(), right.clone()])
            .unwrap_err()
            .code(),
        "VALUE_DOMAIN_CROSS_GRAPH"
    );

    let entries = vec![
        MapValueEntry::new(Value::Text("a".into()), left),
        MapValueEntry::new(Value::Text("b".into()), right),
    ];
    assert_eq!(
        MapValue::new(
            MapKeyType::Text,
            ValueType::domain(DomainType::Item),
            entries,
        )
        .unwrap_err()
        .code(),
        "VALUE_DOMAIN_CROSS_GRAPH"
    );
}

#[test]
fn nominal_values_preserve_graph_affinity_and_cannot_be_declared_inputs() {
    let (registry, pair_id) = pair_registry();
    let first = graph();
    let second = graph();
    let left = handle(&first, 0, DomainType::Project);
    let same = handle(&first, 1, DomainType::Project);
    let other = handle(&second, 1, DomainType::Project);

    let pair = Value::structure(&registry, pair_id, vec![left.clone(), same]).unwrap();
    assert_eq!(
        pair.validate_declared_input(&registry).unwrap_err().code(),
        "VALUE_DOMAIN_PUBLIC_INPUT"
    );
    assert_eq!(
        Value::structure(&registry, pair_id, vec![left.clone(), other])
            .unwrap_err()
            .code(),
        "VALUE_DOMAIN_CROSS_GRAPH"
    );
    assert_eq!(
        left.validate_declared_input(&registry).unwrap_err().code(),
        "VALUE_DOMAIN_PUBLIC_INPUT"
    );
    Value::Integer(1)
        .validate_declared_input(&registry)
        .unwrap();

    let stale = TypeRegistry::default();
    assert_eq!(
        pair.validate_declared_input(&stale).unwrap_err().code(),
        "VALUE_NOMINAL_DEFINITION"
    );
    assert_eq!(
        ValueType::primitive(PrimitiveType::Integer).to_string(),
        "int"
    );
}

#[test]
fn handle_identity_includes_graph_slot_and_domain_type() {
    let first = graph();
    let second = graph();
    assert_ne!(
        handle(&first, 0, DomainType::Layer),
        handle(&second, 0, DomainType::Layer)
    );
    assert_ne!(
        handle(&first, 0, DomainType::Layer),
        handle(&first, 1, DomainType::Layer)
    );
    assert_ne!(
        handle(&first, 0, DomainType::Layer),
        handle(&first, 0, DomainType::Item)
    );
    assert!(format!("{first:?}").contains("<opaque>"));
}
