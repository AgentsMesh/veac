use crate::program::{
    DomainOperationContract, DomainOperationExposure, DomainOperationRegistry, DomainType,
};

use super::*;

pub(super) fn current() -> StandardLibrarySpec {
    let registry = DomainOperationRegistry::standard();
    let mut types = DomainType::all()
        .map(|value| StandardLibraryType {
            name: value.name().to_owned(),
            domain_type_opcode: value.opcode(),
        })
        .collect::<Vec<_>>();
    types.sort_by(|left, right| left.name.cmp(&right.name));

    let mut free_functions = Vec::new();
    let mut methods = Vec::new();
    for operation in registry.contracts() {
        match operation.exposure() {
            DomainOperationExposure::FreeFunction { name } => {
                free_functions.push(function(operation, name));
            }
            DomainOperationExposure::Method { receiver, name } => {
                methods.push(method(operation, *receiver, name));
            }
        }
    }
    free_functions.sort_by(|left, right| left.name.cmp(&right.name));
    methods.sort_by(|left, right| {
        (left.receiver.opcode, left.name.as_str())
            .cmp(&(right.receiver.opcode, right.name.as_str()))
    });

    StandardLibrarySpec {
        domain_opset_version: registry.version().raw(),
        types,
        free_functions,
        methods,
    }
}

fn function(operation: &DomainOperationContract, name: &str) -> StandardLibraryFunction {
    StandardLibraryFunction {
        name: name.to_owned(),
        operation_opcode: operation.id().opcode(),
        contract: super::super::domain_contract::catalog::signature(operation),
    }
}

fn method(
    operation: &DomainOperationContract,
    receiver: DomainType,
    name: &str,
) -> StandardLibraryMethod {
    StandardLibraryMethod {
        receiver: DomainTypeReferenceSpec {
            opcode: receiver.opcode(),
            name: receiver.name().to_owned(),
        },
        name: name.to_owned(),
        operation_opcode: operation.id().opcode(),
        contract: super::super::domain_contract::catalog::signature(operation),
    }
}
