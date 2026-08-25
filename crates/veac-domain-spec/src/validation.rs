use std::collections::BTreeSet;

use veac_lang_model::{Effect, Stage};

use super::{
    catalog, DomainInstructionKind, DomainOperationContract, DomainOperationExposure,
    DomainRegistryError, DomainRuntimeAction, DomainValueShape, OperandAxis,
    MAX_DOMAIN_OPERATION_OPERANDS,
};

#[path = "validation_action.rs"]
mod action;

pub(super) fn contract(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    if value.name() != value.id().name() || !crate::is_name(value.name()) {
        return Err(error(
            "DOMAIN_OPERATION_NAME",
            format!(
                "operation {} has a stale or invalid canonical name",
                value.id()
            ),
        ));
    }
    exposure(value)?;
    if value.operands().len() > MAX_DOMAIN_OPERATION_OPERANDS {
        return Err(error(
            "DOMAIN_OPERATION_OPERAND_LIMIT",
            format!(
                "operation {} exceeds the {MAX_DOMAIN_OPERATION_OPERANDS} operand limit",
                value.name()
            ),
        ));
    }
    operands(value)?;
    instruction(value)?;
    temporal(value)?;
    result(value)?;
    action::validate(value)?;
    if *value != catalog::contract(value.id()) {
        return Err(error(
            "DOMAIN_OPERATION_CONTRACT",
            format!(
                "operation {} does not match its v{} contract",
                value.name(),
                super::DomainOpsetVersion::CURRENT
            ),
        ));
    }
    Ok(())
}

fn exposure(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    if !crate::is_name(value.exposure().name()) {
        return Err(error(
            "DOMAIN_OPERATION_EXPOSURE",
            format!("operation {} has an invalid surface name", value.name()),
        ));
    }
    if let DomainOperationExposure::Method { receiver, .. } = value.exposure() {
        let first = value.operands().first().map(|operand| operand.shape());
        if first != Some(DomainValueShape::Domain(*receiver)) {
            return Err(error(
                "DOMAIN_OPERATION_EXPOSURE",
                format!(
                    "operation {} method receiver does not match its first operand",
                    value.name()
                ),
            ));
        }
    }
    Ok(())
}

fn operands(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let mut names = BTreeSet::new();
    for operand in value.operands() {
        if !crate::is_name(operand.name()) || !names.insert(operand.name()) {
            return Err(error(
                "DOMAIN_OPERATION_OPERAND",
                format!(
                    "operation {} has an invalid or duplicate operand `{}`",
                    value.name(),
                    operand.name()
                ),
            ));
        }
        if operand.axis() == OperandAxis::Leaf
            && operand
                .shape()
                .domain_type()
                .is_some_and(|value| value.requires_topology_axis())
        {
            return Err(error(
                "DOMAIN_OPERATION_AXIS",
                format!(
                    "operation {} sends topology value `{}` through a leaf operand",
                    value.name(),
                    operand.name()
                ),
            ));
        }
    }
    Ok(())
}

fn instruction(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let valid = matches!(
        (value.instruction(), value.runtime_action(), value.effect()),
        (
            DomainInstructionKind::DomainConstruct,
            DomainRuntimeAction::Description,
            Effect::Pure
        ) | (
            DomainInstructionKind::GraphEmit,
            DomainRuntimeAction::EntityConstructor
                | DomainRuntimeAction::OwnedAttachment
                | DomainRuntimeAction::NonOwningUpdate
                | DomainRuntimeAction::ProjectEntry
                | DomainRuntimeAction::RelationConstructor,
            Effect::GraphEmit
        )
    );
    valid.then_some(()).ok_or_else(|| {
        error(
            "DOMAIN_OPERATION_EFFECT",
            format!(
                "operation {} has an effect inconsistent with its instruction kind",
                value.name()
            ),
        )
    })
}

fn result(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let DomainValueShape::Domain(result) = value.result() else {
        return Err(error(
            "DOMAIN_OPERATION_RESULT",
            format!("operation {} must return one domain value", value.name()),
        ));
    };
    let valid = match value.instruction() {
        DomainInstructionKind::DomainConstruct => !result.is_graph_entity(),
        DomainInstructionKind::GraphEmit => {
            result.is_graph_entity()
                && value
                    .operands()
                    .iter()
                    .any(|operand| operand.axis() == OperandAxis::Topology)
        }
    };
    valid.then_some(()).ok_or_else(|| {
        error(
            "DOMAIN_OPERATION_RESULT",
            format!(
                "operation {} has an invalid result for its instruction kind",
                value.name()
            ),
        )
    })
}

fn temporal(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let valid = matches!(
        (value.max_stage(), value.temporal_lowering()),
        (Stage::Build, None) | (Stage::Temporal, Some(_))
    );
    valid.then_some(()).ok_or_else(|| {
        error(
            "DOMAIN_OPERATION_TEMPORAL",
            format!(
                "operation {} has an inconsistent Temporal availability contract",
                value.name()
            ),
        )
    })
}

fn error(code: &'static str, message: impl Into<String>) -> DomainRegistryError {
    DomainRegistryError::new(code, message)
}
