use std::sync::Arc;

use super::super::super::Value;
use crate::program::expression::execution_budget::{
    LOGICAL_NOMINAL_BASE_BYTES, LOGICAL_NOMINAL_FIELD_HANDLE_BYTES,
};
use crate::program::expression::{PrimitiveType, ValueType};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeId, TypeRegistry, TypeRegistryBuilder, VariantIndex,
};

#[test]
fn values_share_verified_layout_handles_and_render_canonically() {
    let (registry, point, status) = registry("label");
    let point_value = Value::structure(
        &registry,
        point,
        vec![Value::Integer(7), Value::Text("north".into())],
    )
    .unwrap();
    let Value::Struct(structure) = &point_value else {
        unreachable!()
    };
    assert!(std::ptr::eq(
        structure.definition(),
        registry.definition(point).unwrap()
    ));
    assert_eq!(point_value.render(), r#"Point { x: 7, label: "north" }"#);
    assert_eq!(
        point_value.retained_bytes(),
        LOGICAL_NOMINAL_BASE_BYTES + 2 * LOGICAL_NOMINAL_FIELD_HANDLE_BYTES + 5
    );
    assert_eq!(point_value.evaluated_bytes(), 0);

    let empty = Value::variant(&registry, status, VariantIndex::new(0), Vec::new()).unwrap();
    let ready = Value::variant(
        &registry,
        status,
        VariantIndex::new(1),
        vec![Value::Text("go".into())],
    )
    .unwrap();
    assert_eq!(empty.render(), "Status.Empty");
    assert_eq!(ready.render(), r#"Status.Ready { label: "go" }"#);
}

#[test]
fn constructors_reject_wrong_kind_variant_arity_and_field_type() {
    let (registry, point, status) = registry("label");
    assert_eq!(
        Value::structure(&registry, status, Vec::new())
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_KIND"
    );
    assert_eq!(
        Value::variant(&registry, point, VariantIndex::new(0), Vec::new())
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_KIND"
    );
    assert_eq!(
        Value::structure(&registry, point, vec![Value::Integer(1)])
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_FIELD_COUNT"
    );
    assert_eq!(
        Value::structure(
            &registry,
            point,
            vec![Value::Text("bad".into()), Value::Text("ok".into())],
        )
        .unwrap_err()
        .code(),
        "VALUE_NOMINAL_FIELD_TYPE"
    );
    assert_eq!(
        Value::variant(&registry, status, VariantIndex::new(9), Vec::new())
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_VARIANT"
    );
}

#[test]
fn stale_nested_layouts_are_rejected_even_when_type_ids_match() {
    let (left, _, _) = nested_registry("left");
    let (right, root, child) = nested_registry("right");
    let stale_child = Value::structure(&left, child, vec![Value::Integer(1)]).unwrap();
    assert_eq!(
        stale_child
            .validate_nominal_registry(&right)
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_DEFINITION"
    );
    assert_eq!(
        Value::structure(&right, root, vec![stale_child])
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_DEFINITION"
    );
}

#[test]
fn empty_structs_keep_explicit_braces() {
    let definition = TypeDefinition::new(
        "types.veac",
        "Marker",
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    );
    let id = definition.type_ref().id();
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(definition)).unwrap();
    let value = Value::structure(&builder.finish().unwrap(), id, Vec::new()).unwrap();
    assert_eq!(value.render(), "Marker {}");
}

fn registry(child_name: &str) -> (TypeRegistry, TypeId, TypeId) {
    let point = TypeDefinition::new(
        "types.veac",
        "Point",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![
            field(0, "x", int()),
            field(1, "label", text()),
        ])),
    );
    let status = TypeDefinition::new(
        "types.veac",
        "Status",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![
            EnumVariantDefinition::new(VariantIndex::new(0), "Empty", Vec::new()),
            EnumVariantDefinition::new(
                VariantIndex::new(1),
                "Ready",
                vec![field(0, child_name, text())],
            ),
        ])),
    );
    finish(vec![point, status])
}

pub(super) fn nested_registry(child_field: &str) -> (TypeRegistry, TypeId, TypeId) {
    let child = TypeDefinition::new(
        "types.veac",
        "Child",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field(0, child_field, int())])),
    );
    let child_id = child.type_ref().id();
    let root = TypeDefinition::new(
        "types.veac",
        "Root",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field(
            0,
            "child",
            ValueType::nominal(child.type_ref().clone()),
        )])),
    );
    let root_id = root.type_ref().id();
    let (registry, _, _) = finish(vec![child, root]);
    (registry, root_id, child_id)
}

fn finish(values: Vec<TypeDefinition>) -> (TypeRegistry, TypeId, TypeId) {
    let ids = (values[0].type_ref().id(), values[1].type_ref().id());
    let mut builder = TypeRegistryBuilder::new();
    for value in values {
        builder.insert(Arc::new(value)).unwrap();
    }
    let registry = builder.finish().unwrap();
    (registry, ids.0, ids.1)
}

fn field(index: u16, name: &str, value_type: ValueType) -> FieldDefinition {
    FieldDefinition::new(FieldIndex::new(index), name, value_type)
}

fn int() -> ValueType {
    ValueType::primitive(PrimitiveType::Integer)
}

fn text() -> ValueType {
    ValueType::primitive(PrimitiveType::Text)
}
