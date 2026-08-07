use std::ops::Range;

use super::definitions::Definitions;
use super::error;
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreProgram, CoreValueMetadata, FunctionRegistry, Stage,
    ValueId,
};
use crate::program::expression::{ExpressionError, ValueType};
use crate::program::{DomainInstructionKind, DomainOperationContract, DomainOperationRegistry};

#[cfg(test)]
mod tests;

pub(super) fn identity(
    program: &CoreProgram,
    registry: &DomainOperationRegistry,
) -> Result<(), ExpressionError> {
    if program.domain_opset != registry.version() {
        return Err(error(
            format!(
                "Core domain opset {} does not match runtime opset {}",
                program.domain_opset,
                registry.version()
            ),
            0..0,
        ));
    }
    (program.domain_registry_digest == registry.digest())
        .then_some(())
        .ok_or_else(|| {
            error(
                "Core domain registry digest does not match the runtime",
                0..0,
            )
        })
}

pub(super) fn called_identities(
    program: &CoreProgram,
    functions: &FunctionRegistry,
) -> Result<(), ExpressionError> {
    for instruction in program.blocks.iter().flat_map(|block| &block.instructions) {
        let CoreInstructionKind::Call {
            target: super::super::CoreCallTarget::User(id),
            ..
        } = instruction.kind
        else {
            continue;
        };
        let Some(function) = functions.get(id) else {
            continue;
        };
        let body = function.body();
        if body.domain_opset != program.domain_opset
            || body.domain_registry_digest != program.domain_registry_digest
        {
            return Err(error(
                "called Core program has a different domain operation identity",
                instruction.span.clone(),
            ));
        }
    }
    Ok(())
}

pub(super) fn instruction(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
    registry: &DomainOperationRegistry,
) -> Result<ValueType, ExpressionError> {
    let (opcode, operands, expected_kind) = match &instruction.kind {
        CoreInstructionKind::DomainConstruct { opcode, operands } => {
            (*opcode, operands, DomainInstructionKind::DomainConstruct)
        }
        CoreInstructionKind::GraphEmit { opcode, operands } => {
            (*opcode, operands, DomainInstructionKind::GraphEmit)
        }
        _ => {
            return Err(error(
                "Core instruction is not a domain operation",
                instruction.span(),
            ))
        }
    };
    let contract = registry.lookup_opcode(opcode).ok_or_else(|| {
        error(
            format!("unknown domain operation opcode 0x{opcode:04x}"),
            instruction.span(),
        )
    })?;
    if contract.instruction() != expected_kind {
        return Err(error(
            "domain opcode uses the wrong Core instruction kind",
            instruction.span(),
        ));
    }
    verify_operands(contract, operands, program, definitions, instruction.span())?;
    let expected_type = contract.result().value_type();
    let actual = program.value_type(instruction.type_id).ok_or_else(|| {
        error(
            "domain instruction result must have a value type",
            instruction.span(),
        )
    })?;
    if actual != &expected_type {
        return Err(error(
            format!("domain instruction declares {actual}, expected {expected_type}"),
            instruction.span(),
        ));
    }
    let metadata = expected_metadata(contract, operands, definitions);
    if instruction.metadata != metadata {
        return Err(error(
            "domain instruction metadata does not match its operation contract",
            instruction.span(),
        ));
    }
    if metadata.shape_stage() > Stage::Build {
        return Err(error(
            "domain topology cannot depend on Temporal-stage data",
            instruction.span(),
        ));
    }
    if metadata.leaf_stage() > contract.max_stage() {
        return Err(error(
            format!(
                "domain operation `{}` is unavailable at Temporal stage",
                contract.name()
            ),
            instruction.span(),
        ));
    }
    Ok(expected_type)
}

pub(super) fn expected_metadata(
    contract: &DomainOperationContract,
    operands: &[ValueId],
    definitions: &Definitions,
) -> CoreValueMetadata {
    CoreValueMetadata::domain(
        contract,
        operands
            .iter()
            .map(|operand| definitions.metadata(*operand)),
    )
}

fn verify_operands(
    contract: &DomainOperationContract,
    operands: &[ValueId],
    program: &CoreProgram,
    definitions: &Definitions,
    span: Range<usize>,
) -> Result<(), ExpressionError> {
    if operands.len() != contract.operands().len() {
        return Err(error("domain operation operand arity mismatch", span));
    }
    for (id, expected) in operands.iter().zip(contract.operands()) {
        let actual = program
            .value_type(definitions.type_id(*id))
            .ok_or_else(|| error("domain operand must have a value type", span.clone()))?;
        if actual != &expected.shape().value_type() {
            return Err(error(
                format!(
                    "domain operand `{}` type does not match its contract",
                    expected.name()
                ),
                span,
            ));
        }
    }
    Ok(())
}
