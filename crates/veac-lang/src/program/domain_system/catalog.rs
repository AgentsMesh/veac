use super::{DomainOperationContract, DomainOperationId};

mod builders;
mod generated;

pub(super) fn contracts() -> Vec<DomainOperationContract> {
    DomainOperationId::all().map(contract).collect()
}

pub(super) fn contract(id: DomainOperationId) -> DomainOperationContract {
    generated::contract(id)
}
