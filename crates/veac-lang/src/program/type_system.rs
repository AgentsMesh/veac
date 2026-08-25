//! Stable nominal type identities and verified declaration-order layouts.

mod syntax;

pub use syntax::{TypeSyntax, TypeSyntaxError, TypeSyntaxKind};
pub use veac_lang_model::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionDigest, TypeDefinitionKind, TypeId, TypeRef, TypeRegistry,
    TypeRegistryBuilder, TypeRegistryError, VariantIndex, MAX_TYPE_DECLARATIONS, MAX_TYPE_MEMBERS,
    MAX_TYPE_REGISTRY_BYTES, MAX_TYPE_REGISTRY_DEFINITIONS,
};
