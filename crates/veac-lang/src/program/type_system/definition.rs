use std::sync::Arc;

use crate::program::expression::ValueType;

use super::digest;
use super::{FieldIndex, TypeDefinitionDigest, TypeId, TypeRef, VariantIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDefinition {
    index: FieldIndex,
    name: Arc<str>,
    value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDefinition {
    fields: Arc<[FieldDefinition]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantDefinition {
    index: VariantIndex,
    name: Arc<str>,
    fields: Arc<[FieldDefinition]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDefinition {
    variants: Arc<[EnumVariantDefinition]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDefinitionKind {
    Struct(StructDefinition),
    Enum(EnumDefinition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinition {
    pub(crate) type_ref: TypeRef,
    pub(crate) canonical_source_id: Arc<str>,
    pub(crate) declared_name: Arc<str>,
    pub(crate) digest: TypeDefinitionDigest,
    pub(crate) kind: TypeDefinitionKind,
}

impl FieldDefinition {
    pub fn new(index: FieldIndex, name: impl Into<Arc<str>>, value_type: ValueType) -> Self {
        Self {
            index,
            name: name.into(),
            value_type,
        }
    }

    pub const fn index(&self) -> FieldIndex {
        self.index
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn value_type(&self) -> &ValueType {
        &self.value_type
    }
}

impl StructDefinition {
    pub fn new(fields: impl Into<Arc<[FieldDefinition]>>) -> Self {
        Self {
            fields: fields.into(),
        }
    }

    pub fn fields(&self) -> &[FieldDefinition] {
        &self.fields
    }

    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields.iter().find(|field| field.name() == name)
    }
}

impl EnumVariantDefinition {
    pub fn new(
        index: VariantIndex,
        name: impl Into<Arc<str>>,
        fields: impl Into<Arc<[FieldDefinition]>>,
    ) -> Self {
        Self {
            index,
            name: name.into(),
            fields: fields.into(),
        }
    }

    pub const fn index(&self) -> VariantIndex {
        self.index
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[FieldDefinition] {
        &self.fields
    }

    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields.iter().find(|field| field.name() == name)
    }
}

impl EnumDefinition {
    pub fn new(variants: impl Into<Arc<[EnumVariantDefinition]>>) -> Self {
        Self {
            variants: variants.into(),
        }
    }

    pub fn variants(&self) -> &[EnumVariantDefinition] {
        &self.variants
    }

    pub fn variant(&self, name: &str) -> Option<&EnumVariantDefinition> {
        self.variants.iter().find(|variant| variant.name() == name)
    }
}

impl TypeDefinition {
    pub fn new(
        canonical_source_id: impl Into<Arc<str>>,
        declared_name: impl Into<Arc<str>>,
        kind: TypeDefinitionKind,
    ) -> Self {
        let canonical_source_id = canonical_source_id.into();
        let declared_name = declared_name.into();
        let id = TypeId::derive(&canonical_source_id, &declared_name);
        let digest = digest::definition(&kind);
        Self {
            type_ref: TypeRef::new(id, Arc::clone(&declared_name)),
            canonical_source_id,
            declared_name,
            digest,
            kind,
        }
    }

    pub const fn type_ref(&self) -> &TypeRef {
        &self.type_ref
    }

    pub fn canonical_source_id(&self) -> &str {
        &self.canonical_source_id
    }

    pub fn declared_name(&self) -> &str {
        &self.declared_name
    }

    pub const fn digest(&self) -> TypeDefinitionDigest {
        self.digest
    }

    pub const fn kind(&self) -> &TypeDefinitionKind {
        &self.kind
    }

    pub fn retained_bytes(&self) -> Option<usize> {
        super::retained::definition(self)
    }
}
