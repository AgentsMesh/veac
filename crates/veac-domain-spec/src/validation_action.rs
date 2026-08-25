use veac_lang_model::PrimitiveType;

use crate::{
    DomainOperandContract, DomainOperationContract, DomainOperationExposure, DomainRegistryError,
    DomainRuntimeAction, DomainType, DomainValueShape, OperandAxis, TemporalLoweringOpcode,
};

pub(super) fn validate(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    match value.runtime_action() {
        DomainRuntimeAction::Description => temporal_shape(value),
        DomainRuntimeAction::EntityConstructor => entity_constructor(value),
        DomainRuntimeAction::OwnedAttachment => owned_attachment(value),
        DomainRuntimeAction::NonOwningUpdate => non_owning_update(value),
        DomainRuntimeAction::ProjectEntry => project_entry(value),
        DomainRuntimeAction::RelationConstructor => relation_constructor(value),
    }
}

fn temporal_shape(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let Some(lowering) = value.temporal_lowering() else {
        return Ok(());
    };
    let (result, operands) = match lowering {
        TemporalLoweringOpcode::ComposePoint => (
            DomainType::Point,
            [PrimitiveType::Length, PrimitiveType::Length].as_slice(),
        ),
        TemporalLoweringOpcode::ComposeVector => (
            DomainType::Vector,
            [PrimitiveType::Scalar, PrimitiveType::Scalar].as_slice(),
        ),
        TemporalLoweringOpcode::ComposeRect => (
            DomainType::Rect,
            [
                PrimitiveType::Scalar,
                PrimitiveType::Scalar,
                PrimitiveType::Scalar,
                PrimitiveType::Scalar,
            ]
            .as_slice(),
        ),
    };
    let valid = value.result() == DomainValueShape::Domain(result)
        && value.operands().len() == operands.len()
        && value
            .operands()
            .iter()
            .zip(operands)
            .all(|(operand, expected)| {
                operand.axis() == OperandAxis::Leaf
                    && operand.shape() == DomainValueShape::Primitive(*expected)
            });
    valid
        .then_some(())
        .ok_or_else(|| shape_error(value, "temporal lowering"))
}

fn entity_constructor(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let valid = matches!(
        value.exposure(),
        DomainOperationExposure::FreeFunction { .. }
    ) && value
        .result()
        .domain_type()
        .is_some_and(DomainType::is_graph_entity)
        && value.operands().first().is_some_and(identifier_key);
    valid
        .then_some(())
        .ok_or_else(|| shape_error(value, "entity constructor"))
}

fn owned_attachment(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let Some(receiver) = value.exposure().receiver() else {
        return Err(shape_error(value, "owned attachment"));
    };
    let valid = receiver.is_container()
        && value.result() == DomainValueShape::Domain(receiver)
        && value.operands().len() == 2
        && value.operands()[0].shape() == DomainValueShape::Domain(receiver)
        && value.operands()[0].axis() == OperandAxis::Topology
        && value.operands()[1].axis() == OperandAxis::Topology
        && value.operands()[1]
            .shape()
            .domain_type()
            .is_some_and(DomainType::is_graph_entity);
    valid
        .then_some(())
        .ok_or_else(|| shape_error(value, "owned attachment"))
}

fn non_owning_update(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let Some(receiver) = value.exposure().receiver() else {
        return Err(shape_error(value, "non-owning update"));
    };
    let valid = receiver.is_graph_entity()
        && value.result() == DomainValueShape::Domain(receiver)
        && value.operands().len() == 2
        && value.operands()[0].shape() == DomainValueShape::Domain(receiver)
        && value.operands()[0].axis() == OperandAxis::Topology
        && !value.operands()[1]
            .shape()
            .domain_type()
            .is_some_and(DomainType::is_graph_entity);
    valid
        .then_some(())
        .ok_or_else(|| shape_error(value, "non-owning update"))
}

fn project_entry(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let valid = value.exposure().receiver() == Some(DomainType::Project)
        && value.result() == DomainValueShape::Domain(DomainType::Project)
        && value.operands() == project_entry_operands().as_slice();
    valid
        .then_some(())
        .ok_or_else(|| shape_error(value, "project entry"))
}

fn relation_constructor(value: &DomainOperationContract) -> Result<(), DomainRegistryError> {
    let valid = matches!(
        value.exposure(),
        DomainOperationExposure::FreeFunction { .. }
    ) && value.result() == DomainValueShape::Domain(DomainType::Relation)
        && value.operands().first().is_some_and(identifier_key)
        && value.operands().iter().skip(1).any(|operand| {
            operand.axis() == OperandAxis::Topology
                && operand
                    .shape()
                    .domain_type()
                    .is_some_and(DomainType::is_graph_entity)
        });
    valid
        .then_some(())
        .ok_or_else(|| shape_error(value, "relation constructor"))
}

fn project_entry_operands() -> [DomainOperandContract; 2] {
    [
        DomainOperandContract::new(
            "project",
            DomainValueShape::Domain(DomainType::Project),
            OperandAxis::Topology,
        ),
        DomainOperandContract::new(
            "sequence",
            DomainValueShape::Domain(DomainType::Sequence),
            OperandAxis::Topology,
        ),
    ]
}

fn identifier_key(operand: &DomainOperandContract) -> bool {
    operand.axis() == OperandAxis::Topology
        && operand.shape() == DomainValueShape::Primitive(PrimitiveType::Identifier)
}

fn shape_error(value: &DomainOperationContract, kind: &str) -> DomainRegistryError {
    DomainRegistryError::new(
        "DOMAIN_OPERATION_ACTION_SHAPE",
        format!("operation {} has an invalid {kind} shape", value.name()),
    )
}
