use std::mem::{size_of, size_of_val};

use super::{CompiledFunction, CoreInstructionKind, CoreProgram, CoreType};
use crate::program::expression::{ValueType, ValueTypeKind};

mod metadata;
mod verified;

impl CompiledFunction {
    pub(crate) fn retained_bytes(&self) -> Option<usize> {
        let inline_body = size_of::<super::VerifiedCoreProgram>();
        let mut bytes = size_of::<Self>()
            .checked_sub(inline_body)?
            .checked_add(self.name.len())?;
        if let Some(origin) = &self.origin {
            bytes = bytes.checked_add(origin.source_id().len())?;
        }
        for parameter in &self.parameters {
            bytes = bytes
                .checked_add(size_of_val(parameter))?
                .checked_add(parameter.name.len())?
                .checked_add(value_type_bytes(&parameter.value_type)?)?;
        }
        bytes = bytes.checked_add(value_type_bytes(&self.return_type)?)?;
        bytes = bytes.checked_add(metadata::summary_payload(&self.summary)?)?;
        bytes.checked_add(self.body.retained_bytes()?)
    }
}

impl CoreProgram {
    fn retained_bytes(&self) -> Option<usize> {
        let bytes = self
            .types
            .entries()
            .iter()
            .try_fold(size_of::<Self>(), |bytes, entry| {
                let payload = match entry.kind() {
                    CoreType::Value(value) => value_type_bytes(value)?,
                    CoreType::MapBuilder { .. } | CoreType::MapPending { .. } => 0,
                };
                bytes.checked_add(size_of_val(entry))?.checked_add(payload)
            })?;
        let bytes = self
            .nominal_definitions
            .iter()
            .try_fold(bytes, |bytes, entry| {
                bytes
                    .checked_add(size_of_val(entry))?
                    .checked_add(entry.definition().retained_bytes()?)
            })?;
        let bytes = self.inputs.iter().try_fold(bytes, |bytes, input| {
            let bytes = bytes
                .checked_add(size_of_val(input))?
                .checked_add(input.name.len())?;
            match &input.callable {
                Some(value) => bytes
                    .checked_add(sequence_type_bytes(&value.capture_types)?)?
                    .checked_add(metadata::summary_payload(&value.summary)?),
                None => Some(bytes),
            }
        })?;
        let bytes = self.local_slots.iter().try_fold(bytes, |bytes, slot| {
            bytes
                .checked_add(size_of_val(slot))?
                .checked_add(metadata::payload(&slot.metadata)?)
        })?;
        let bytes = self
            .closure_definitions
            .iter()
            .try_fold(bytes, closure_definition_bytes)?;
        self.blocks.iter().try_fold(bytes, |bytes, block| {
            let mut bytes = bytes.checked_add(size_of_val(block))?;
            bytes = bytes.checked_add(
                block
                    .parameters
                    .len()
                    .checked_mul(size_of::<super::CoreBlockParameter>())?,
            )?;
            for parameter in &block.parameters {
                bytes = bytes.checked_add(metadata::payload(&parameter.metadata)?)?;
            }
            for instruction in &block.instructions {
                bytes = bytes.checked_add(size_of_val(instruction))?;
                bytes = bytes.checked_add(instruction_payload(&instruction.kind)?)?;
                bytes = bytes.checked_add(metadata::payload(&instruction.metadata)?)?;
            }
            match &block.terminator {
                super::CoreTerminator::Jump { arguments, .. } => {
                    bytes.checked_add(arguments.len().checked_mul(size_of::<super::ValueId>())?)
                }
                super::CoreTerminator::Match { arms, .. } => {
                    bytes.checked_add(arms.len().checked_mul(size_of::<super::CoreMatchArm>())?)
                }
                super::CoreTerminator::ForEach(value) => bytes
                    .checked_add(
                        value
                            .captures()
                            .len()
                            .checked_mul(size_of::<super::ValueId>())?,
                    )?
                    .checked_add(metadata::payload(value.result_metadata())?),
                _ => Some(bytes),
            }
        })
    }
}

fn instruction_payload(instruction: &CoreInstructionKind) -> Option<usize> {
    let value_ids = match instruction {
        CoreInstructionKind::Literal(value) => return Some(value.retained_bytes()),
        CoreInstructionKind::Closure { captures, .. } => captures.len(),
        CoreInstructionKind::TemporalAttach { selectors, .. } => selectors.len().checked_add(2)?,
        CoreInstructionKind::Invoke { arguments, .. }
        | CoreInstructionKind::Call { arguments, .. } => arguments.len(),
        CoreInstructionKind::List { elements }
        | CoreInstructionKind::Tuple { elements }
        | CoreInstructionKind::StructConstruct {
            fields: elements, ..
        }
        | CoreInstructionKind::EnumConstruct {
            fields: elements, ..
        }
        | CoreInstructionKind::DomainConstruct {
            operands: elements, ..
        }
        | CoreInstructionKind::GraphEmit {
            operands: elements, ..
        }
        | CoreInstructionKind::TemporalCompose {
            operands: elements, ..
        } => elements.len(),
        CoreInstructionKind::Input(_)
        | CoreInstructionKind::Parameter(_)
        | CoreInstructionKind::Capture(_)
        | CoreInstructionKind::LocalInit { .. }
        | CoreInstructionKind::LocalSet { .. }
        | CoreInstructionKind::LocalGet { .. }
        | CoreInstructionKind::Unary { .. }
        | CoreInstructionKind::Arithmetic { .. }
        | CoreInstructionKind::Compare { .. }
        | CoreInstructionKind::Equal { .. }
        | CoreInstructionKind::Collection { .. }
        | CoreInstructionKind::StructProject { .. }
        | CoreInstructionKind::TemporalProject { .. }
        | CoreInstructionKind::Range { .. }
        | CoreInstructionKind::MapBegin { .. }
        | CoreInstructionKind::MapKey { .. }
        | CoreInstructionKind::MapValue { .. }
        | CoreInstructionKind::MapFinish { .. } => 0,
    };
    value_ids.checked_mul(size_of::<super::ValueId>())
}

fn closure_definition_bytes(
    bytes: usize,
    definition: &super::CoreClosureDefinition,
) -> Option<usize> {
    bytes
        .checked_add(size_of_val(definition))?
        .checked_add(sequence_type_bytes(&definition.parameter_types)?)?
        .checked_add(
            definition
                .parameter_stages
                .len()
                .checked_mul(size_of::<super::Stage>())?,
        )?
        .checked_add(sequence_type_bytes(&definition.capture_types)?)?
        .checked_add(metadata::summary_payload(&definition.summary)?)?
        .checked_add(definition.body.retained_bytes()?)
}

fn value_type_bytes(value: &ValueType) -> Option<usize> {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Some(0),
        ValueTypeKind::Nominal(value) => Some(value.diagnostic_name().len()),
        ValueTypeKind::List(element)
        | ValueTypeKind::Range(element)
        | ValueTypeKind::Map { value: element, .. } => {
            size_of::<ValueType>().checked_add(value_type_bytes(element)?)
        }
        ValueTypeKind::Tuple(elements) => sequence_type_bytes(elements),
        ValueTypeKind::Function {
            parameters, result, ..
        } => sequence_type_bytes(parameters)?
            .checked_add(size_of::<ValueType>())?
            .checked_add(value_type_bytes(result)?),
    }
}

fn sequence_type_bytes(values: &[ValueType]) -> Option<usize> {
    values.iter().try_fold(
        values.len().checked_mul(size_of::<ValueType>())?,
        |bytes, value| bytes.checked_add(value_type_bytes(value)?),
    )
}

#[cfg(test)]
#[path = "retained/tests.rs"]
mod tests;
