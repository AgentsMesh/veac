//! Stable, execution-free identities and types shared by VEAC language layers.

mod domain_type;
mod effect;
mod name;
mod nominal;
mod primitive;
mod stage;
mod value_type;

#[cfg(test)]
mod tests;

pub use domain_type::DomainType;
pub use effect::{Effect, FunctionEffect};
pub use nominal::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionDigest, TypeDefinitionKind, TypeId, TypeRef, TypeRegistry,
    TypeRegistryBuilder, TypeRegistryError, VariantIndex, MAX_TYPE_DECLARATIONS, MAX_TYPE_MEMBERS,
    MAX_TYPE_REGISTRY_BYTES, MAX_TYPE_REGISTRY_DEFINITIONS,
};
pub use primitive::PrimitiveType;
pub use stage::Stage;
pub use value_type::{
    MapKeyType, ValueType, ValueTypeError, ValueTypeKind, ValueTypeParseError,
    MAX_VALUE_TYPE_ARITY, MAX_VALUE_TYPE_DEPTH,
};
