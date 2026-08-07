use crate::program::expression::PrimitiveType;

use super::super::{
    DomainOperandContract, DomainOperationContract, DomainOperationExposure, DomainOperationId,
    DomainOperationSemantics, DomainRuntimeAction, DomainType, DomainValueShape, OperandAxis,
};

pub(super) fn build(
    id: DomainOperationId,
    exposure: DomainOperationExposure,
    operands: Vec<DomainOperandContract>,
    result: DomainType,
    action: DomainRuntimeAction,
) -> DomainOperationContract {
    let semantics = match action {
        DomainRuntimeAction::Description => DomainOperationSemantics::description(),
        action => DomainOperationSemantics::graph(action),
    };
    DomainOperationContract::new(id, id.name(), exposure, operands, domain(result), semantics)
}

pub(super) fn free(name: &'static str) -> DomainOperationExposure {
    DomainOperationExposure::free_function(name)
}

pub(super) fn method(receiver: DomainType, name: &'static str) -> DomainOperationExposure {
    DomainOperationExposure::method(receiver, name)
}

pub(super) fn topology(name: &'static str, shape: DomainValueShape) -> DomainOperandContract {
    DomainOperandContract::new(name, shape, OperandAxis::Topology)
}

pub(super) fn leaf(name: &'static str, shape: DomainValueShape) -> DomainOperandContract {
    DomainOperandContract::new(name, shape, OperandAxis::Leaf)
}

pub(super) const fn primitive(value: PrimitiveType) -> DomainValueShape {
    DomainValueShape::primitive(value)
}

pub(super) const fn primitive_list(value: PrimitiveType) -> DomainValueShape {
    DomainValueShape::primitive_list(value)
}

pub(super) const fn domain(value: DomainType) -> DomainValueShape {
    DomainValueShape::domain(value)
}

pub(super) const fn domain_list(value: DomainType) -> DomainValueShape {
    DomainValueShape::domain_list(value)
}
