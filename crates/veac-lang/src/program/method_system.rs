//! Statically selected nominal methods and their immutable signatures.

mod definition;
mod error;
mod identity;
mod registry;
mod retained;

pub use definition::{MethodBody, MethodDefinition, MethodSignature, MethodVisibility};
pub use error::MethodRegistryError;
pub use registry::{Builder as MethodRegistryBuilder, MethodRegistry};

pub const MAX_METHODS_PER_TYPE: usize = 64;
pub const MAX_METHOD_REGISTRY_DEFINITIONS: usize = 1_024;
pub const MAX_METHOD_REGISTRY_BYTES: usize = 8 * 1024 * 1024;

pub(crate) fn retained_bytes(value: &MethodDefinition) -> Option<usize> {
    retained::definition(value)
}

#[cfg(test)]
mod tests;
