//! Stable nominal identities and verified declaration-order layouts.

mod definition;
mod digest;
mod error;
mod identity;
mod index;
mod registry;
mod retained;

pub use definition::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, StructDefinition, TypeDefinition,
    TypeDefinitionKind,
};
pub use error::TypeRegistryError;
pub use identity::{TypeDefinitionDigest, TypeId, TypeRef};
pub use index::{FieldIndex, VariantIndex};
pub use registry::{Builder as TypeRegistryBuilder, TypeRegistry};

pub const MAX_TYPE_DECLARATIONS: usize = 256;
pub const MAX_TYPE_MEMBERS: usize = 64;
pub const MAX_TYPE_REGISTRY_DEFINITIONS: usize = 1024;
pub const MAX_TYPE_REGISTRY_BYTES: usize = 8 * 1024 * 1024;

#[cfg(test)]
mod tests;
