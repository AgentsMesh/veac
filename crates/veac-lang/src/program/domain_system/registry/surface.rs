use super::DomainOperationRegistry;
use crate::program::{DomainOperationContract, DomainType};

impl DomainOperationRegistry {
    pub fn lookup_function(&self, name: &str) -> Option<&DomainOperationContract> {
        self.lookup(*self.functions.get(name)?)
    }

    pub fn lookup_method(
        &self,
        receiver: DomainType,
        name: &str,
    ) -> Option<&DomainOperationContract> {
        self.lookup(*self.methods.get(&receiver)?.get(name)?)
    }
}
