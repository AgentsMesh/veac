use std::collections::BTreeSet;

use crate::program::{DomainOperationId, DomainOperationRegistry, DomainType};

use super::*;

pub(super) fn validate(value: &DomainOpsetSpec) -> Result<(), VocabularyValidationError> {
    validate_contracts(value)?;
    validate_identity(value, identity::calculate(value)?)
}

pub(super) fn validate_with_plugins(
    value: &DomainOpsetSpec,
    plugins: &[super::super::PluginEffectSpec],
) -> Result<(), VocabularyValidationError> {
    validate_contracts(value)?;
    let digest = identity::calculate_with_plugins(
        value,
        &super::super::StandardLibrarySpec::current(),
        plugins,
    )?;
    validate_identity(value, digest)
}

fn validate_contracts(value: &DomainOpsetSpec) -> Result<(), VocabularyValidationError> {
    let registry = DomainOperationRegistry::standard();
    if value.version != registry.version().raw() {
        return Err(error("domain opset version does not match this VEAC build"));
    }
    validate_types(value)?;
    validate_operations(value)
}

fn validate_identity(
    value: &DomainOpsetSpec,
    calculated_digest: String,
) -> Result<(), VocabularyValidationError> {
    let registry = DomainOperationRegistry::standard();
    if calculated_digest != value.registry_digest {
        return Err(error(
            "domain opset registry digest does not match its contracts",
        ));
    }
    if value.registry_digest != registry.digest().to_string() {
        return Err(error(
            "domain opset registry identity does not match this VEAC build",
        ));
    }
    if value != &catalog::current() {
        return Err(error(
            "domain opset does not match the current closed registry",
        ));
    }
    Ok(())
}

fn validate_types(value: &DomainOpsetSpec) -> Result<(), VocabularyValidationError> {
    if value.types.len() != DomainType::all().len() {
        return Err(error(
            "domain opset does not publish every closed domain type",
        ));
    }
    if value
        .types
        .windows(2)
        .any(|pair| pair[0].opcode >= pair[1].opcode)
    {
        return Err(error("domain types are not unique and ordered by opcode"));
    }
    let names = value
        .types
        .iter()
        .map(|item| item.name.as_str())
        .collect::<BTreeSet<_>>();
    if names.len() != value.types.len() {
        return Err(error("domain type names are not unique"));
    }
    for item in &value.types {
        let Some(runtime) = DomainType::from_opcode(item.opcode) else {
            return Err(error(format!("unknown domain type opcode {}", item.opcode)));
        };
        if item.name != runtime.name() || item.container != runtime.is_container() {
            return Err(error(format!(
                "domain type {} has a stale contract",
                item.opcode
            )));
        }
    }
    Ok(())
}

fn validate_operations(value: &DomainOpsetSpec) -> Result<(), VocabularyValidationError> {
    if value.operations.len() != DomainOperationId::all().len() {
        return Err(error("domain opset does not publish every operation"));
    }
    if value
        .operations
        .windows(2)
        .any(|pair| pair[0].opcode >= pair[1].opcode)
    {
        return Err(error(
            "domain operations are not unique and ordered by opcode",
        ));
    }
    let names = value
        .operations
        .iter()
        .map(|item| item.name.as_str())
        .collect::<BTreeSet<_>>();
    if names.len() != value.operations.len() {
        return Err(error("domain operation names are not unique"));
    }
    let registry = DomainOperationRegistry::standard();
    for item in &value.operations {
        let Some(contract) = registry.lookup_opcode(item.opcode) else {
            return Err(error(format!(
                "unknown domain operation opcode {}",
                item.opcode
            )));
        };
        if item.name != contract.name() || item.contract != catalog::signature(contract) {
            return Err(error(format!(
                "domain operation {} has a stale contract",
                item.opcode
            )));
        }
    }
    Ok(())
}

fn error(message: impl Into<String>) -> VocabularyValidationError {
    VocabularyValidationError::new(message)
}
