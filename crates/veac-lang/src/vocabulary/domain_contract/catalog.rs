use crate::program::expression::{Effect, PrimitiveType, Stage};
use crate::program::{
    DomainInstructionKind, DomainOperationContract, DomainOperationRegistry, DomainRuntimeAction,
    DomainType, DomainValueShape, OperandAxis, TemporalLoweringOpcode,
};

use super::*;

pub(super) fn current() -> DomainOpsetSpec {
    let registry = DomainOperationRegistry::standard();
    let mut types = DomainType::all().map(domain_type).collect::<Vec<_>>();
    types.sort_by_key(|value| value.opcode);
    let mut operations = registry.contracts().map(operation).collect::<Vec<_>>();
    operations.sort_by_key(|value| value.opcode);
    DomainOpsetSpec {
        version: registry.version().raw(),
        registry_digest: registry.digest().to_string(),
        types,
        operations,
    }
}

pub(crate) fn signature(value: &DomainOperationContract) -> DomainOperationSignature {
    DomainOperationSignature {
        ordered_operands: value
            .operands()
            .iter()
            .map(|operand| DomainOperandSpec {
                name: operand.name().to_owned(),
                shape: shape(operand.shape()),
                axis: axis(operand.axis()),
            })
            .collect(),
        result: shape(value.result()),
        instruction: instruction(value.instruction()),
        runtime_action: runtime_action(value.runtime_action()),
        effect: effect(value.effect()),
        max_stage: max_stage(value.max_stage()),
        temporal_lowering: value.temporal_lowering().map(temporal_lowering),
    }
}

fn domain_type(value: DomainType) -> DomainTypeSpec {
    DomainTypeSpec {
        opcode: value.opcode(),
        name: value.name().to_owned(),
        container: value.is_container(),
    }
}

fn operation(value: &DomainOperationContract) -> DomainOperationSpec {
    DomainOperationSpec {
        opcode: value.id().opcode(),
        name: value.name().to_owned(),
        contract: signature(value),
    }
}

fn shape(value: DomainValueShape) -> DomainValueShapeSpec {
    match value {
        DomainValueShape::Primitive(value) => DomainValueShapeSpec::Primitive {
            name: primitive_name(value).to_owned(),
        },
        DomainValueShape::PrimitiveList(value) => DomainValueShapeSpec::PrimitiveList {
            name: primitive_name(value).to_owned(),
        },
        DomainValueShape::Domain(value) => reference(value, false),
        DomainValueShape::DomainList(value) => reference(value, true),
    }
}

fn reference(value: DomainType, list: bool) -> DomainValueShapeSpec {
    if list {
        DomainValueShapeSpec::DomainList {
            type_opcode: value.opcode(),
            type_name: value.name().to_owned(),
        }
    } else {
        DomainValueShapeSpec::Domain {
            type_opcode: value.opcode(),
            type_name: value.name().to_owned(),
        }
    }
}

fn primitive_name(value: PrimitiveType) -> &'static str {
    value.as_str()
}

fn axis(value: OperandAxis) -> DomainOperandAxisSpec {
    match value {
        OperandAxis::Topology => DomainOperandAxisSpec::Topology,
        OperandAxis::Leaf => DomainOperandAxisSpec::Leaf,
    }
}

fn instruction(value: DomainInstructionKind) -> DomainInstructionSpec {
    match value {
        DomainInstructionKind::DomainConstruct => DomainInstructionSpec::DomainConstruct,
        DomainInstructionKind::GraphEmit => DomainInstructionSpec::GraphEmit,
    }
}

fn runtime_action(value: DomainRuntimeAction) -> DomainRuntimeActionSpec {
    match value {
        DomainRuntimeAction::Description => DomainRuntimeActionSpec::Description,
        DomainRuntimeAction::EntityConstructor => DomainRuntimeActionSpec::EntityConstructor,
        DomainRuntimeAction::OwnedAttachment => DomainRuntimeActionSpec::OwnedAttachment,
        DomainRuntimeAction::NonOwningUpdate => DomainRuntimeActionSpec::NonOwningUpdate,
        DomainRuntimeAction::ProjectEntry => DomainRuntimeActionSpec::ProjectEntry,
        DomainRuntimeAction::RelationConstructor => DomainRuntimeActionSpec::RelationConstructor,
    }
}

fn effect(value: Effect) -> DomainEffectSpec {
    match value {
        Effect::Pure => DomainEffectSpec::Pure,
        Effect::LocalMutation => DomainEffectSpec::LocalMutation,
        Effect::GraphEmit => DomainEffectSpec::GraphEmit,
    }
}

fn max_stage(value: Stage) -> DomainMaxStageSpec {
    match value {
        Stage::Build => DomainMaxStageSpec::Build,
        Stage::Temporal => DomainMaxStageSpec::Temporal,
        Stage::Const => unreachable!("domain operation maximum stage is never Const"),
    }
}

fn temporal_lowering(value: TemporalLoweringOpcode) -> TemporalLoweringOpcodeSpec {
    match value {
        TemporalLoweringOpcode::ComposeVector => TemporalLoweringOpcodeSpec::Vector,
        TemporalLoweringOpcode::ComposePoint => TemporalLoweringOpcodeSpec::Point,
        TemporalLoweringOpcode::ComposeRect => TemporalLoweringOpcodeSpec::Rect,
    }
}
