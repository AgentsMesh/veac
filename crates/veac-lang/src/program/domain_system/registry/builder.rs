use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::{digest, retained, validation};
use super::{
    DomainOperationContract, DomainOperationExposure, DomainOperationId, DomainOperationRegistry,
    DomainOpsetVersion, DomainRegistryError, DomainType,
};

pub(super) struct Builder {
    version: DomainOpsetVersion,
    contracts: BTreeMap<DomainOperationId, Arc<DomainOperationContract>>,
    names: BTreeMap<String, DomainOperationId>,
    functions: BTreeMap<String, DomainOperationId>,
    methods: BTreeMap<DomainType, BTreeMap<String, DomainOperationId>>,
    retained_bytes: usize,
    count_limit: usize,
    retained_limit: usize,
}

impl Builder {
    pub(super) fn new(
        version: DomainOpsetVersion,
        count_limit: usize,
        retained_limit: usize,
    ) -> Self {
        Self {
            version,
            contracts: BTreeMap::new(),
            names: BTreeMap::new(),
            functions: BTreeMap::new(),
            methods: BTreeMap::new(),
            retained_bytes: 0,
            count_limit,
            retained_limit,
        }
    }

    pub(super) fn insert(
        &mut self,
        value: Arc<DomainOperationContract>,
    ) -> Result<(), DomainRegistryError> {
        if self.contracts.contains_key(&value.id()) {
            return Err(error(
                "DOMAIN_OPERATION_DUPLICATE_ID",
                format!("operation ID {} occurs more than once", value.id()),
            ));
        }
        if self.names.contains_key(value.name()) {
            return Err(error(
                "DOMAIN_OPERATION_DUPLICATE_NAME",
                format!("operation name `{}` occurs more than once", value.name()),
            ));
        }
        if self.contracts.len() >= self.count_limit {
            return Err(limit("domain operation registry exceeds its count limit"));
        }
        self.reject_duplicate_exposure(&value)?;
        validation::contract(&value)?;
        let retained = retained::contract(&value)
            .ok_or_else(|| limit("domain operation retained storage overflows"))?;
        self.retained_bytes = self
            .retained_bytes
            .checked_add(retained)
            .filter(|value| *value <= self.retained_limit)
            .ok_or_else(|| limit("domain operation registry exceeds its retained limit"))?;
        self.names.insert(value.name().to_owned(), value.id());
        match value.exposure() {
            DomainOperationExposure::FreeFunction { name } => {
                self.functions.insert(name.to_string(), value.id());
            }
            DomainOperationExposure::Method { receiver, name } => {
                self.methods
                    .entry(*receiver)
                    .or_default()
                    .insert(name.to_string(), value.id());
            }
        }
        self.contracts.insert(value.id(), value);
        Ok(())
    }

    pub(super) fn finish(self) -> Result<DomainOperationRegistry, DomainRegistryError> {
        for id in DomainOperationId::all() {
            if !self.contracts.contains_key(&id) {
                return Err(error(
                    "DOMAIN_OPERATION_MISSING",
                    format!("domain operation {} `{}` is missing", id, id.name()),
                ));
            }
        }
        let digest = digest::registry(self.version, self.contracts.values().map(Arc::as_ref));
        Ok(DomainOperationRegistry {
            version: self.version,
            contracts: self.contracts,
            names: self.names,
            functions: self.functions,
            methods: self.methods,
            retained_bytes: self.retained_bytes,
            digest,
        })
    }

    fn reject_duplicate_exposure(
        &self,
        value: &DomainOperationContract,
    ) -> Result<(), DomainRegistryError> {
        let duplicate = match value.exposure() {
            DomainOperationExposure::FreeFunction { name } => {
                self.functions.contains_key(name.as_ref())
            }
            DomainOperationExposure::Method { receiver, name } => self
                .methods
                .get(receiver)
                .is_some_and(|methods| methods.contains_key(name.as_ref())),
        };
        if duplicate {
            return Err(error(
                "DOMAIN_OPERATION_DUPLICATE_EXPOSURE",
                format!(
                    "standard-library exposure `{}` occurs more than once",
                    value.exposure().name()
                ),
            ));
        }
        Ok(())
    }
}

fn limit(message: &'static str) -> DomainRegistryError {
    error("DOMAIN_OPERATION_REGISTRY_LIMIT", message)
}

fn error(code: &'static str, message: impl Into<String>) -> DomainRegistryError {
    DomainRegistryError::new(code, message)
}

#[cfg(test)]
#[path = "builder/tests.rs"]
mod tests;
