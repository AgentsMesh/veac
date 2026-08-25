use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use super::{
    catalog, DomainOperationContract, DomainOperationExposure, DomainOperationId,
    DomainOpsetVersion, DomainRegistryDigest, DomainRegistryError, DomainType,
    MAX_DOMAIN_OPERATIONS, MAX_DOMAIN_REGISTRY_BYTES,
};

mod builder;
mod surface;

use builder::Builder;

#[derive(Debug, Clone)]
pub struct DomainOperationRegistry {
    version: DomainOpsetVersion,
    contracts: BTreeMap<DomainOperationId, Arc<DomainOperationContract>>,
    names: BTreeMap<String, DomainOperationId>,
    functions: BTreeMap<String, DomainOperationId>,
    methods: BTreeMap<DomainType, BTreeMap<String, DomainOperationId>>,
    retained_bytes: usize,
    digest: DomainRegistryDigest,
}

impl DomainOperationRegistry {
    pub fn standard() -> Self {
        Self::shared().clone()
    }

    pub fn shared() -> &'static Self {
        static STANDARD: OnceLock<DomainOperationRegistry> = OnceLock::new();
        STANDARD.get_or_init(|| {
            Self::for_version(DomainOpsetVersion::CURRENT)
                .expect("the built-in domain operation catalog is verified")
        })
    }

    pub fn for_version(version: DomainOpsetVersion) -> Result<Self, DomainRegistryError> {
        if version != DomainOpsetVersion::CURRENT {
            return Err(DomainRegistryError::new(
                "DOMAIN_OPSET_UNSUPPORTED",
                format!("domain operation set version {version} is not supported"),
            ));
        }
        build(
            version,
            catalog::contracts(),
            MAX_DOMAIN_OPERATIONS,
            MAX_DOMAIN_REGISTRY_BYTES,
        )
    }

    pub const fn version(&self) -> DomainOpsetVersion {
        self.version
    }

    pub fn lookup(&self, id: DomainOperationId) -> Option<&DomainOperationContract> {
        self.contracts.get(&id).map(Arc::as_ref)
    }

    pub fn lookup_opcode(&self, opcode: u16) -> Option<&DomainOperationContract> {
        self.lookup(DomainOperationId::from_opcode(opcode)?)
    }

    pub fn lookup_name(&self, name: &str) -> Option<&DomainOperationContract> {
        self.lookup(*self.names.get(name)?)
    }

    pub fn contracts(&self) -> impl ExactSizeIterator<Item = &DomainOperationContract> {
        self.contracts.values().map(Arc::as_ref)
    }

    pub fn len(&self) -> usize {
        self.contracts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contracts.is_empty()
    }

    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub const fn digest(&self) -> DomainRegistryDigest {
        self.digest
    }
}

impl Default for DomainOperationRegistry {
    fn default() -> Self {
        Self::standard()
    }
}

pub(super) fn build(
    version: DomainOpsetVersion,
    contracts: impl IntoIterator<Item = DomainOperationContract>,
    count_limit: usize,
    retained_limit: usize,
) -> Result<DomainOperationRegistry, DomainRegistryError> {
    let mut builder = Builder::new(version, count_limit, retained_limit);
    for contract in contracts {
        builder.insert(Arc::new(contract))?;
    }
    builder.finish()
}
