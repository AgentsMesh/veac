use super::*;
use crate::program::{TypeId, TypeRef};
use std::collections::BTreeSet;

#[test]
fn default_builder_finishes_as_an_empty_registry() {
    let registry = MethodRegistryBuilder::default().finish();
    assert!(registry.is_empty());
    assert_eq!(registry.definitions().count(), 0);
    assert_eq!(registry.function_ids().count(), 0);
}

#[test]
fn builder_admits_lookup_merge_and_idempotent_definitions() {
    let types = types("types.veac", "Brand");
    let value = method(&types, "title", MethodVisibility::Exported, true);
    let mut builder = MethodRegistryBuilder::new();
    builder.insert(Arc::clone(&value), &types).unwrap();
    builder.insert(Arc::clone(&value), &types).unwrap();
    let registry = builder.finish();
    let receiver = types.resolve("Brand").unwrap().id();
    assert_eq!(registry.lookup(receiver, "title"), Some(value.as_ref()));

    let mut merged = MethodRegistryBuilder::new();
    merged.merge(&registry, &types).unwrap();
    assert_eq!(merged.finish().len(), 1);
}

#[test]
fn admission_recomputes_id_owner_and_receiver_contracts() {
    let types = types("types.veac", "Brand");
    let mut stale = method(&types, "title", MethodVisibility::Private, false)
        .as_ref()
        .clone();
    super::super::definition::tests_support::replace_function_id(
        &mut stale,
        crate::program::expression::FunctionId::from_bytes([7; 32]),
    );
    assert_eq!(insert_error(stale, &types), "METHOD_ID_MISMATCH");

    let foreign = MethodDefinition::new(
        method(&types, "title", MethodVisibility::Private, false)
            .signature()
            .clone(),
        "other.veac",
        MethodVisibility::Private,
    );
    assert_eq!(insert_error(foreign, &types), "METHOD_FOREIGN_OWNER");

    let unknown_types = TypeRegistry::default();
    let unknown = method(&types, "title", MethodVisibility::Private, false)
        .as_ref()
        .clone();
    assert_eq!(insert_error(unknown, &unknown_types), "METHOD_UNKNOWN_TYPE");
}

#[test]
fn duplicate_names_and_retained_boundaries_fail_closed() {
    let types = types("types.veac", "Brand");
    let first = method(&types, "title", MethodVisibility::Private, false);
    let receiver = types.resolve("Brand").unwrap().clone();
    let changed = Arc::new(MethodDefinition::new(
        MethodSignature::new(
            receiver,
            "title",
            Vec::new(),
            ValueType::primitive(PrimitiveType::Integer),
        ),
        "types.veac",
        MethodVisibility::Private,
    ));
    let mut duplicates = MethodRegistryBuilder::new();
    duplicates.insert(first, &types).unwrap();
    assert_eq!(
        duplicates.insert(changed, &types).unwrap_err().code(),
        "METHOD_DUPLICATE"
    );

    let value = method(&types, "exact", MethodVisibility::Private, true);
    let exact = super::super::retained::definition(&value).unwrap();
    let mut accepted = MethodRegistryBuilder::with_limit(exact);
    accepted.insert(Arc::clone(&value), &types).unwrap();
    assert_eq!(accepted.finish().retained_bytes(), exact);
    let mut rejected = MethodRegistryBuilder::with_limit(exact - 1);
    assert_eq!(
        rejected.insert(value, &types).unwrap_err().code(),
        "METHOD_REGISTRY_LIMIT"
    );
}

#[test]
fn exported_and_reachable_snapshots_are_closed_and_stable() {
    let types = types("types.veac", "Brand");
    let public = method(&types, "title", MethodVisibility::Exported, true);
    let private = method(&types, "helper", MethodVisibility::Private, true);
    let mut builder = MethodRegistryBuilder::new();
    builder.insert(Arc::clone(&public), &types).unwrap();
    builder.insert(Arc::clone(&private), &types).unwrap();
    let registry = builder.finish();
    let receiver = types.resolve("Brand").unwrap().id();

    let exported = registry.exported_for(&BTreeSet::from([receiver]));
    assert_eq!(exported.len(), 1);
    assert!(exported.lookup(receiver, "title").unwrap().body().is_none());
    assert!(exported.lookup(receiver, "helper").is_none());

    let reachable = registry.reachable([private.signature().function_id()]);
    assert_eq!(reachable.len(), 1);
    assert!(reachable
        .lookup(receiver, "helper")
        .unwrap()
        .body()
        .is_some());
}

#[test]
fn signature_names_types_origins_and_per_type_limits_are_verified() {
    let types = types("types.veac", "Brand");
    let receiver = types.resolve("Brand").unwrap().clone();
    let invalid_name = MethodDefinition::new(
        MethodSignature::new(
            receiver.clone(),
            "bad.name",
            Vec::new(),
            ValueType::primitive(PrimitiveType::Text),
        ),
        "types.veac",
        MethodVisibility::Private,
    );
    assert_eq!(insert_error(invalid_name, &types), "METHOD_SIGNATURE");

    let missing = TypeRef::new(TypeId::derive("missing.veac", "Missing"), "Missing");
    let unknown_type = MethodDefinition::new(
        MethodSignature::new(
            receiver,
            "unknown",
            vec![FunctionParameter::new(
                "value",
                ValueType::list(ValueType::nominal(missing)).unwrap(),
            )],
            ValueType::primitive(PrimitiveType::Text),
        ),
        "types.veac",
        MethodVisibility::Private,
    );
    assert_eq!(insert_error(unknown_type, &types), "METHOD_UNKNOWN_TYPE");

    let origin = method(&types, "origin", MethodVisibility::Private, false)
        .as_ref()
        .clone()
        .with_body(MethodBody::new(
            "{}",
            FunctionOrigin::new("other.veac", 0..2),
        ));
    assert_eq!(insert_error(origin, &types), "METHOD_BODY_ORIGIN");

    let mut builder = MethodRegistryBuilder::new();
    for index in 0..MAX_METHODS_PER_TYPE {
        builder
            .insert(
                method(
                    &types,
                    &format!("method_{index}"),
                    MethodVisibility::Private,
                    false,
                ),
                &types,
            )
            .unwrap();
    }
    let error = builder
        .insert(
            method(&types, "overflow", MethodVisibility::Private, false),
            &types,
        )
        .unwrap_err();
    assert_eq!(error.code(), "METHOD_REGISTRY_LIMIT");
}

fn insert_error(value: MethodDefinition, types: &TypeRegistry) -> &'static str {
    MethodRegistryBuilder::new()
        .insert(Arc::new(value), types)
        .unwrap_err()
        .code()
}
