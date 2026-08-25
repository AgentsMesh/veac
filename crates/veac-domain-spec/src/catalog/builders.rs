use veac_lang_model::PrimitiveType;

use super::super::{
    DomainInstructionKind, DomainOperandContract, DomainOperationContract, DomainOperationExposure,
    DomainOperationId, DomainOperationSemantics, DomainRuntimeAction, DomainType, DomainValueShape,
    OperandAxis, TemporalLoweringOpcode,
};
use veac_lang_model::{Effect, Stage};

pub(super) fn build(
    id: DomainOperationId,
    exposure: DomainOperationExposure,
    operands: Vec<DomainOperandContract>,
    result: DomainType,
    semantics: DomainOperationSemantics,
) -> DomainOperationContract {
    DomainOperationContract::new(id, id.name(), exposure, operands, domain(result), semantics)
}

pub(super) const fn semantics(
    instruction: DomainInstructionKind,
    runtime_action: DomainRuntimeAction,
    effect: Effect,
    max_stage: Stage,
    temporal_lowering: Option<TemporalLoweringOpcode>,
) -> DomainOperationSemantics {
    DomainOperationSemantics::new(
        instruction,
        runtime_action,
        effect,
        max_stage,
        temporal_lowering,
    )
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
