//! Facade for the execution-free closed domain specification.
//!
//! Domain identities and contracts live in `veac-domain-spec`; this module is
//! intentionally limited to the public path used by compiler clients.

pub use veac_domain_spec::{
    DomainInstructionKind, DomainOperandContract, DomainOperationContract, DomainOperationExposure,
    DomainOperationId, DomainOperationRegistry, DomainOpsetVersion, DomainPluginIdentity,
    DomainRegistryDigest, DomainRegistryError, DomainRuntimeAction, DomainType, DomainValueShape,
    OperandAxis, TemporalLoweringOpcode, MAX_DOMAIN_OPERATIONS, MAX_DOMAIN_OPERATION_OPERANDS,
    MAX_DOMAIN_REGISTRY_BYTES,
};
