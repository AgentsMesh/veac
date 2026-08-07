use sha2::{Digest, Sha256};

use super::*;

const DIGEST_DOMAIN: &[u8] = b"veac.domain-operation-registry.v5\0";

mod plugin;
mod surface;

pub(super) fn calculate(value: &DomainOpsetSpec) -> Result<String, VocabularyValidationError> {
    calculate_with_plugins(
        value,
        &super::super::StandardLibrarySpec::current(),
        &super::super::plugin_effects::current(),
    )
}

pub(super) fn calculate_with_plugins(
    value: &DomainOpsetSpec,
    library: &super::super::StandardLibrarySpec,
    plugins: &[super::super::PluginEffectSpec],
) -> Result<String, VocabularyValidationError> {
    let exposures = surface::collect(library)?;
    if exposures.len() != value.operations.len() {
        return Err(error(
            "standard-library exposure count does not match opset",
        ));
    }
    let mut digest = Sha256::new();
    digest.update(DIGEST_DOMAIN);
    digest.update(value.version.to_be_bytes());
    digest.update((value.operations.len() as u64).to_be_bytes());
    for operation in &value.operations {
        digest.update(operation.opcode.to_be_bytes());
        framed(&mut digest, operation.name.as_bytes());
        surface::update(
            &mut digest,
            exposures
                .get(&operation.opcode)
                .ok_or_else(|| error("domain operation has no standard-library exposure"))?,
        );
        digest.update([
            instruction(operation.contract.instruction),
            runtime_action(operation.contract.runtime_action),
            effect(operation.contract.effect),
            max_stage(operation.contract.max_stage),
        ]);
        temporal_lowering(&mut digest, operation.contract.temporal_lowering);
        digest.update((operation.contract.ordered_operands.len() as u64).to_be_bytes());
        for operand in &operation.contract.ordered_operands {
            framed(&mut digest, operand.name.as_bytes());
            shape(&mut digest, &operand.shape)?;
            digest.update([axis(operand.axis)]);
        }
        shape(&mut digest, &operation.contract.result)?;
    }
    plugin::update(&mut digest, plugins);
    Ok(format!("{:x}", digest.finalize()))
}

fn shape(
    digest: &mut Sha256,
    value: &DomainValueShapeSpec,
) -> Result<(), VocabularyValidationError> {
    match value {
        DomainValueShapeSpec::Primitive { name } => {
            digest.update([0x00, primitive(name)?]);
        }
        DomainValueShapeSpec::PrimitiveList { name } => {
            digest.update([0x03, primitive(name)?]);
        }
        DomainValueShapeSpec::Domain { type_opcode, .. } => {
            digest.update([0x01]);
            digest.update(type_opcode.to_be_bytes());
        }
        DomainValueShapeSpec::DomainList { type_opcode, .. } => {
            digest.update([0x02]);
            digest.update(type_opcode.to_be_bytes());
        }
    }
    Ok(())
}

fn primitive(value: &str) -> Result<u8, VocabularyValidationError> {
    match value {
        "int" => Ok(0x00),
        "scalar" => Ok(0x01),
        "time" => Ok(0x02),
        "length" => Ok(0x03),
        "percent" => Ok(0x04),
        "angle" => Ok(0x05),
        "text" => Ok(0x06),
        "color" => Ok(0x07),
        "bool" => Ok(0x08),
        "identifier" => Ok(0x09),
        _ => Err(error(format!("unknown primitive domain shape `{value}`"))),
    }
}

const fn instruction(value: DomainInstructionSpec) -> u8 {
    match value {
        DomainInstructionSpec::DomainConstruct => 0x00,
        DomainInstructionSpec::GraphEmit => 0x01,
    }
}

const fn runtime_action(value: DomainRuntimeActionSpec) -> u8 {
    match value {
        DomainRuntimeActionSpec::Description => 0x00,
        DomainRuntimeActionSpec::EntityConstructor => 0x01,
        DomainRuntimeActionSpec::OwnedAttachment => 0x02,
        DomainRuntimeActionSpec::NonOwningUpdate => 0x03,
        DomainRuntimeActionSpec::ProjectEntry => 0x04,
        DomainRuntimeActionSpec::RelationConstructor => 0x05,
    }
}

const fn effect(value: DomainEffectSpec) -> u8 {
    match value {
        DomainEffectSpec::Pure => 0x00,
        DomainEffectSpec::LocalMutation => 0x01,
        DomainEffectSpec::GraphEmit => 0x02,
    }
}

const fn axis(value: DomainOperandAxisSpec) -> u8 {
    match value {
        DomainOperandAxisSpec::Topology => 0x00,
        DomainOperandAxisSpec::Leaf => 0x01,
    }
}

const fn max_stage(value: DomainMaxStageSpec) -> u8 {
    match value {
        DomainMaxStageSpec::Build => 0x00,
        DomainMaxStageSpec::Temporal => 0x01,
    }
}

fn temporal_lowering(digest: &mut Sha256, value: Option<TemporalLoweringOpcodeSpec>) {
    let bytes = match value {
        None => &[0x00][..],
        Some(TemporalLoweringOpcodeSpec::Vector) => &[0x01, 0x00],
        Some(TemporalLoweringOpcodeSpec::Point) => &[0x01, 0x01],
        Some(TemporalLoweringOpcodeSpec::Rect) => &[0x01, 0x02],
    };
    digest.update(bytes);
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn error(message: impl Into<String>) -> VocabularyValidationError {
    VocabularyValidationError::new(message)
}
