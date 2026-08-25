//! Closed, versioned domain operation specifications.
//!
//! This crate owns the machine-generated operation vocabulary and its
//! executable-free contracts. Compiler crates consume it directly or through
//! `veac-lang`; runtime consumes only lowered, versioned contracts.

mod catalog;
mod contract;
mod digest;
mod error;
mod name;
mod operation_id;
mod registry;
mod retained;
mod shape;
mod validation;
mod version;

pub(crate) use contract::DomainOperationSemantics;
pub use contract::{
    DomainInstructionKind, DomainOperandContract, DomainOperationContract, DomainOperationExposure,
    DomainRuntimeAction, OperandAxis, TemporalLoweringOpcode,
};
pub use digest::{DomainPluginIdentity, DomainRegistryDigest};
pub use error::DomainRegistryError;
pub use operation_id::DomainOperationId;
pub use registry::DomainOperationRegistry;
pub use shape::DomainValueShape;
pub use veac_lang_model::DomainType;
pub use version::DomainOpsetVersion;

pub const MAX_DOMAIN_OPERATION_OPERANDS: usize = 16;
pub const MAX_DOMAIN_OPERATIONS: usize = 1024;
pub const MAX_DOMAIN_REGISTRY_BYTES: usize = 1024 * 1024;

pub(crate) use name::is_name;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
