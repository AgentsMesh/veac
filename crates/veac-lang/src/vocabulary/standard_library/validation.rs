use std::collections::BTreeSet;

use crate::program::{DomainOperationRegistry, DomainType};

use super::*;

pub(super) fn validate(
    value: &StandardLibrarySpec,
    opset: &super::super::DomainOpsetSpec,
) -> Result<(), VocabularyValidationError> {
    if value.domain_opset_version != opset.version {
        return Err(error("standard library targets a different domain opset"));
    }
    canonical_order(value)?;
    validate_types(value)?;
    let registry = DomainOperationRegistry::standard();
    let mut mapped = BTreeSet::new();
    for function in &value.free_functions {
        validate_function(&registry, function, opset, &mut mapped)?;
    }
    for method in &value.methods {
        validate_method(&registry, method, opset, &mut mapped)?;
    }
    let expected = registry
        .contracts()
        .map(|operation| operation.id().opcode())
        .collect::<BTreeSet<_>>();
    if mapped != expected {
        return Err(error(
            "standard library does not map every domain operation exactly once",
        ));
    }
    if value != &catalog::current() {
        return Err(error(
            "standard library does not match the current closed catalog",
        ));
    }
    Ok(())
}

fn validate_types(value: &StandardLibrarySpec) -> Result<(), VocabularyValidationError> {
    if value.types.len() != DomainType::all().len() {
        return Err(error("standard library does not publish every domain type"));
    }
    for item in &value.types {
        let Some(domain_type) = DomainType::from_opcode(item.domain_type_opcode) else {
            return Err(error("standard library type has an unknown domain opcode"));
        };
        if item.name != domain_type.name() {
            return Err(error("standard library type name and opcode disagree"));
        }
    }
    Ok(())
}

fn canonical_order(value: &StandardLibrarySpec) -> Result<(), VocabularyValidationError> {
    if value
        .types
        .windows(2)
        .any(|pair| pair[0].name >= pair[1].name)
    {
        return Err(error(
            "standard library types are not unique and ordered by name",
        ));
    }
    if value
        .free_functions
        .windows(2)
        .any(|pair| pair[0].name >= pair[1].name)
    {
        return Err(error(
            "standard functions are not unique and ordered by name",
        ));
    }
    if value.methods.windows(2).any(|pair| {
        (pair[0].receiver.opcode, pair[0].name.as_str())
            >= (pair[1].receiver.opcode, pair[1].name.as_str())
    }) {
        return Err(error(
            "standard methods are not unique and canonically ordered",
        ));
    }
    Ok(())
}

fn validate_function(
    registry: &DomainOperationRegistry,
    value: &StandardLibraryFunction,
    opset: &super::super::DomainOpsetSpec,
    mapped: &mut BTreeSet<u16>,
) -> Result<(), VocabularyValidationError> {
    let Some(operation) = registry.lookup_function(&value.name) else {
        return Err(error(format!("unknown standard function `{}`", value.name)));
    };
    validate_call(
        value.operation_opcode,
        &value.contract,
        operation.id().opcode(),
        opset,
        mapped,
    )
}

fn validate_method(
    registry: &DomainOperationRegistry,
    value: &StandardLibraryMethod,
    opset: &super::super::DomainOpsetSpec,
    mapped: &mut BTreeSet<u16>,
) -> Result<(), VocabularyValidationError> {
    let Some(receiver) = DomainType::from_opcode(value.receiver.opcode) else {
        return Err(error("standard method has an unknown receiver opcode"));
    };
    if value.receiver.name != receiver.name() {
        return Err(error("standard method receiver name and opcode disagree"));
    }
    let Some(operation) = registry.lookup_method(receiver, &value.name) else {
        return Err(error(format!("unknown standard method `{}`", value.name)));
    };
    validate_call(
        value.operation_opcode,
        &value.contract,
        operation.id().opcode(),
        opset,
        mapped,
    )
}

fn validate_call(
    opcode: u16,
    signature: &super::super::DomainOperationSignature,
    expected_opcode: u16,
    opset: &super::super::DomainOpsetSpec,
    mapped: &mut BTreeSet<u16>,
) -> Result<(), VocabularyValidationError> {
    let operation = opset.operations.iter().find(|value| value.opcode == opcode);
    if opcode != expected_opcode || operation.map(|value| &value.contract) != Some(signature) {
        return Err(error(
            "standard callable does not match its numeric operation contract",
        ));
    }
    if !mapped.insert(opcode) {
        return Err(error(
            "domain operation is mapped by more than one standard callable",
        ));
    }
    Ok(())
}

fn error(message: impl Into<String>) -> VocabularyValidationError {
    VocabularyValidationError::new(message)
}
