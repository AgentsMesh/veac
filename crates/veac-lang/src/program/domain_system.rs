//! Closed, versioned contracts for executable video-domain operations.

mod catalog;
mod contract;
mod digest;
mod domain_type;
mod error;
mod operation_id;
mod registry;
mod retained;
mod shape;
mod validation;
mod version;

pub(super) use contract::DomainOperationSemantics;
pub use contract::{
    DomainInstructionKind, DomainOperandContract, DomainOperationContract, DomainOperationExposure,
    DomainRuntimeAction, OperandAxis, TemporalLoweringOpcode,
};
pub use digest::DomainRegistryDigest;
pub use domain_type::DomainType;
pub use error::DomainRegistryError;
pub use operation_id::DomainOperationId;
pub use registry::DomainOperationRegistry;
pub use shape::DomainValueShape;
pub use version::DomainOpsetVersion;

pub const MAX_DOMAIN_OPERATION_OPERANDS: usize = 16;
pub const MAX_DOMAIN_OPERATIONS: usize = 1024;
pub const MAX_DOMAIN_REGISTRY_BYTES: usize = 1024 * 1024;

#[cfg(test)]
#[path = "domain_system/tests.rs"]
mod tests;
