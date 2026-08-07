use super::super::super::*;
use crate::program::expression::{PrimitiveType, ValueType};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, TypeDefinition,
    TypeDefinitionKind, VariantIndex,
};

pub(super) fn choice_definition() -> TypeDefinition {
    TypeDefinition::new(
        "types.veac",
        "Choice",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![
            EnumVariantDefinition::new(
                VariantIndex::new(0),
                "Number",
                vec![field(0, "value", integer())],
            ),
            EnumVariantDefinition::new(
                VariantIndex::new(1),
                "Label",
                vec![field(0, "value", text())],
            ),
            EnumVariantDefinition::new(VariantIndex::new(2), "Empty", Vec::new()),
        ])),
    )
}

pub(super) fn table(values: Vec<ValueType>) -> CoreTypeTable {
    CoreTypeTable::new(
        values
            .into_iter()
            .enumerate()
            .map(|(id, value)| CoreTypeEntry {
                id: CoreTypeId::new(id as u32),
                kind: CoreType::Value(value),
            })
            .collect(),
    )
}

pub(super) fn field(index: u16, name: &str, value_type: ValueType) -> FieldDefinition {
    FieldDefinition::new(FieldIndex::new(index), name, value_type)
}

pub(super) fn integer() -> ValueType {
    ValueType::primitive(PrimitiveType::Integer)
}

pub(super) fn text() -> ValueType {
    ValueType::primitive(PrimitiveType::Text)
}

pub(super) fn integer_range() -> ValueType {
    ValueType::range(integer()).unwrap()
}
