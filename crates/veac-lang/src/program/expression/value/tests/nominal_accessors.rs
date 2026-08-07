use std::sync::Arc;

use crate::program::expression::Value;
use crate::program::{
    EnumDefinition, EnumVariantDefinition, StructDefinition, TypeDefinition, TypeDefinitionKind,
    TypeRegistry, TypeRegistryBuilder, VariantIndex,
};

#[test]
fn nominal_accessors_and_equality_use_stable_type_and_variant_identity() {
    let (registry, structure, enumeration) = make_registry("identity.veac", false);
    let left = Value::structure(&registry, structure, Vec::new()).unwrap();
    let right = Value::structure(&registry, structure, Vec::new()).unwrap();
    assert_eq!(left, right);
    let Value::Struct(structure_value) = left else {
        unreachable!()
    };
    assert_eq!(structure_value.type_id(), structure);
    assert_eq!(
        structure_value.definition_digest(),
        registry.definition(structure).unwrap().digest()
    );

    let empty = Value::variant(&registry, enumeration, VariantIndex::new(0), Vec::new()).unwrap();
    let ready = Value::variant(&registry, enumeration, VariantIndex::new(1), Vec::new()).unwrap();
    assert_ne!(empty, ready);
    let Value::Enum(enum_value) = empty else {
        unreachable!()
    };
    assert_eq!(enum_value.type_id(), enumeration);
    assert_eq!(enum_value.variant(), VariantIndex::new(0));
    assert_eq!(
        enum_value.definition_digest(),
        registry.definition(enumeration).unwrap().digest()
    );
}

#[test]
fn unknown_nominal_ids_and_stale_kind_changes_fail_closed() {
    let (registry, _, _) = make_registry("known.veac", false);
    let ghost = TypeDefinition::new(
        "ghost.veac",
        "Ghost",
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    );
    assert_eq!(
        Value::structure(&registry, ghost.type_ref().id(), Vec::new())
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_UNKNOWN_TYPE"
    );

    let (struct_registry, structure, _) = make_registry("changed.veac", false);
    let struct_value = Value::structure(&struct_registry, structure, Vec::new()).unwrap();
    let (enum_registry, replacement, _) = make_registry("changed.veac", true);
    assert_eq!(structure, replacement);
    assert_eq!(
        struct_value
            .validate_nominal_registry(&enum_registry)
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_DEFINITION"
    );

    let enum_value = Value::variant(
        &enum_registry,
        replacement,
        VariantIndex::new(0),
        Vec::new(),
    )
    .unwrap();
    assert_eq!(
        enum_value
            .validate_nominal_registry(&struct_registry)
            .unwrap_err()
            .code(),
        "VALUE_NOMINAL_DEFINITION"
    );
}

fn make_registry(
    path: &str,
    replace_struct_with_enum: bool,
) -> (TypeRegistry, crate::program::TypeId, crate::program::TypeId) {
    let structure_kind = if replace_struct_with_enum {
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Empty",
            Vec::new(),
        )]))
    } else {
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new()))
    };
    let structure = TypeDefinition::new(path, "Marker", structure_kind);
    let enumeration = TypeDefinition::new(
        path,
        "Status",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![
            EnumVariantDefinition::new(VariantIndex::new(0), "Empty", Vec::new()),
            EnumVariantDefinition::new(VariantIndex::new(1), "Ready", Vec::new()),
        ])),
    );
    let ids = (structure.type_ref().id(), enumeration.type_ref().id());
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(structure)).unwrap();
    builder.insert(Arc::new(enumeration)).unwrap();
    (builder.finish().unwrap(), ids.0, ids.1)
}
