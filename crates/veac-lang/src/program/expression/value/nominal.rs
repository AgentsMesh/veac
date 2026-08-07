use std::sync::Arc;

use super::{Value, ValueConstructionError};
use crate::program::expression::execution_budget::{
    LOGICAL_NOMINAL_BASE_BYTES, LOGICAL_NOMINAL_FIELD_HANDLE_BYTES,
};
use crate::program::{
    TypeDefinition, TypeDefinitionDigest, TypeDefinitionKind, TypeId, TypeRegistry, VariantIndex,
};

mod validation;
pub(crate) use validation::validate_registry;

#[derive(Debug, Clone)]
pub struct StructValue {
    definition: Arc<TypeDefinition>,
    fields: Arc<[Value]>,
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    definition: Arc<TypeDefinition>,
    variant: VariantIndex,
    fields: Arc<[Value]>,
}

impl StructValue {
    pub(crate) fn new(
        registry: &TypeRegistry,
        type_id: TypeId,
        fields: Vec<Value>,
    ) -> Result<Self, ValueConstructionError> {
        let definition = definition(registry, type_id)?;
        let TypeDefinitionKind::Struct(layout) = definition.kind() else {
            return Err(kind("struct"));
        };
        validation::validate_fields(registry, layout.fields(), &fields)?;
        Ok(Self {
            definition,
            fields: fields.into(),
        })
    }

    pub fn type_id(&self) -> TypeId {
        self.definition.type_ref().id()
    }

    pub fn definition_digest(&self) -> TypeDefinitionDigest {
        self.definition.digest()
    }

    pub fn fields(&self) -> &[Value] {
        &self.fields
    }

    pub(crate) fn definition(&self) -> &TypeDefinition {
        &self.definition
    }

    pub(crate) fn value_type(&self) -> crate::program::expression::ValueType {
        crate::program::expression::ValueType::nominal(self.definition.type_ref().clone())
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        retained(&self.fields)
    }
}

impl EnumValue {
    pub(crate) fn new(
        registry: &TypeRegistry,
        type_id: TypeId,
        variant: VariantIndex,
        fields: Vec<Value>,
    ) -> Result<Self, ValueConstructionError> {
        let definition = definition(registry, type_id)?;
        let TypeDefinitionKind::Enum(layout) = definition.kind() else {
            return Err(kind("enum"));
        };
        let variant_layout = layout.variants().get(variant.index()).ok_or_else(|| {
            ValueConstructionError::new(
                "VALUE_NOMINAL_VARIANT",
                "enum variant index is outside its verified layout",
            )
        })?;
        validation::validate_fields(registry, variant_layout.fields(), &fields)?;
        Ok(Self {
            definition,
            variant,
            fields: fields.into(),
        })
    }

    pub fn type_id(&self) -> TypeId {
        self.definition.type_ref().id()
    }

    pub fn definition_digest(&self) -> TypeDefinitionDigest {
        self.definition.digest()
    }

    pub fn variant(&self) -> VariantIndex {
        self.variant
    }

    pub fn fields(&self) -> &[Value] {
        &self.fields
    }

    pub(crate) fn definition(&self) -> &TypeDefinition {
        &self.definition
    }

    pub(crate) fn value_type(&self) -> crate::program::expression::ValueType {
        crate::program::expression::ValueType::nominal(self.definition.type_ref().clone())
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        retained(&self.fields)
    }
}

impl PartialEq for StructValue {
    fn eq(&self, other: &Self) -> bool {
        self.type_id() == other.type_id() && self.fields == other.fields
    }
}

impl Eq for StructValue {}

impl PartialEq for EnumValue {
    fn eq(&self, other: &Self) -> bool {
        self.type_id() == other.type_id()
            && self.variant == other.variant
            && self.fields == other.fields
    }
}

impl Eq for EnumValue {}

fn definition(
    registry: &TypeRegistry,
    type_id: TypeId,
) -> Result<Arc<TypeDefinition>, ValueConstructionError> {
    registry.definition_handle(type_id).ok_or_else(|| {
        ValueConstructionError::new(
            "VALUE_NOMINAL_UNKNOWN_TYPE",
            "nominal value refers to a type outside the verified registry",
        )
    })
}

fn retained(fields: &[Value]) -> usize {
    fields
        .iter()
        .fold(LOGICAL_NOMINAL_BASE_BYTES, |bytes, value| {
            bytes
                .saturating_add(LOGICAL_NOMINAL_FIELD_HANDLE_BYTES)
                .saturating_add(value.retained_bytes())
        })
}

fn kind(expected: &str) -> ValueConstructionError {
    ValueConstructionError::new(
        "VALUE_NOMINAL_KIND",
        format!("nominal value expects a {expected} definition"),
    )
}
