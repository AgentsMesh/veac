use std::sync::Arc;

use crate::program::expression::{PrimitiveType, Value, ValueLookup, ValueType};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeRegistry, TypeRegistryBuilder, VariantIndex,
};

use super::*;

#[test]
fn enum_binding_uses_declared_nominal_layout_and_variant_sensitive_digest() {
    let (declarations, types) = fixture(TypeDefinitionKind::Enum(EnumDefinition::new(vec![
        variant(0, "zh-Hans", Vec::new()),
        variant(1, "en", Vec::new()),
    ])));
    let zh = verified(&declarations, &types, "zh-Hans").unwrap();
    let en = verified(&declarations, &types, "en").unwrap();
    assert_ne!(zh.digest(), en.digest());
    let declaration = &declarations["locale"];
    let Value::Enum(value) = ValueLookup::build_value(&en, &declaration.id(), "locale").unwrap()
    else {
        panic!("locale must bind as a nominal enum")
    };
    assert_eq!(value.type_id(), types.resolve("Locale").unwrap().id());
    assert_eq!(value.variant(), VariantIndex::new(1));
}

#[test]
fn unknown_variant_non_enum_and_payload_enum_fail_closed() {
    let (declarations, types) = fixture(TypeDefinitionKind::Enum(EnumDefinition::new(vec![
        variant(0, "zh-Hans", Vec::new()),
    ])));
    assert_error(
        &declarations,
        &types,
        "Missing",
        "PROGRAM_INPUT_ENUM_VARIANT",
    );

    let structure = StructDefinition::new(Vec::<FieldDefinition>::new());
    let (declarations, types) = fixture(TypeDefinitionKind::Struct(structure));
    assert_error(
        &declarations,
        &types,
        "zh-Hans",
        "PROGRAM_INPUT_NOMINAL_KIND",
    );

    let payload = FieldDefinition::new(
        FieldIndex::new(0),
        "region",
        ValueType::from(PrimitiveType::Text),
    );
    let (declarations, types) = fixture(TypeDefinitionKind::Enum(EnumDefinition::new(vec![
        variant(0, "zh-Hans", vec![payload]),
    ])));
    assert_error(
        &declarations,
        &types,
        "zh-Hans",
        "PROGRAM_INPUT_ENUM_PAYLOAD",
    );
}

fn fixture(kind: TypeDefinitionKind) -> (BTreeMap<String, BuildInputDeclaration>, TypeRegistry) {
    let definition = Arc::new(TypeDefinition::new("types.veac", "Locale", kind));
    let reference = definition.type_ref().clone();
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(definition).unwrap();
    builder.bind("Locale", reference.clone()).unwrap();
    let types = builder.finish().unwrap();
    let declaration = BuildInputDeclaration::new(
        "main.veac",
        "locale".to_owned(),
        BuildInputRole::Parameter,
        ValueType::nominal(reference),
    );
    (BTreeMap::from([("locale".to_owned(), declaration)]), types)
}

fn variant(index: u16, name: &str, fields: Vec<FieldDefinition>) -> EnumVariantDefinition {
    EnumVariantDefinition::new(VariantIndex::new(index), name, fields)
}

fn verified(
    declarations: &BTreeMap<String, BuildInputDeclaration>,
    types: &TypeRegistry,
    value: &str,
) -> Result<VerifiedBuildInputs, BuildInputsError> {
    VerifiedBuildInputs::bind(
        declarations,
        types,
        &manifest(vec![binding(
            "locale",
            BuildInputManifestValue::Enum {
                value: value.to_owned(),
            },
        )]),
    )
}

fn assert_error(
    declarations: &BTreeMap<String, BuildInputDeclaration>,
    types: &TypeRegistry,
    value: &str,
    code: &str,
) {
    let error = match verified(declarations, types, value) {
        Ok(_) => panic!("enum binding unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(error.code(), code);
}
