use std::sync::Arc;

use super::{enumeration, structure};
use crate::program::expression::value::{EnumValue, StructValue};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, StructDefinition, TypeDefinition, TypeDefinitionKind,
    TypeRegistryBuilder, VariantIndex,
};

#[test]
fn corrupted_nominal_kinds_and_variant_indexes_fail_closed() {
    let structure_definition = Arc::new(TypeDefinition::new(
        "corrupt.veac",
        "Record",
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    ));
    let enum_definition = Arc::new(TypeDefinition::new(
        "corrupt.veac",
        "Choice",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Empty",
            Vec::new(),
        )])),
    ));
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&structure_definition)).unwrap();
    builder.insert(Arc::clone(&enum_definition)).unwrap();
    let registry = builder.finish().unwrap();

    let corrupt_structure = StructValue {
        definition: Arc::clone(&enum_definition),
        fields: Vec::new().into(),
    };
    assert_eq!(
        structure(&corrupt_structure, &registry).unwrap_err().code(),
        "VALUE_NOMINAL_KIND"
    );
    let corrupt_enum = EnumValue {
        definition: Arc::clone(&structure_definition),
        variant: VariantIndex::new(0),
        fields: Vec::new().into(),
    };
    assert_eq!(
        enumeration(&corrupt_enum, &registry).unwrap_err().code(),
        "VALUE_NOMINAL_KIND"
    );
    let invalid_variant = EnumValue {
        definition: enum_definition,
        variant: VariantIndex::new(9),
        fields: Vec::new().into(),
    };
    assert_eq!(
        enumeration(&invalid_variant, &registry).unwrap_err().code(),
        "VALUE_NOMINAL_VARIANT"
    );
}
