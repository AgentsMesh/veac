use super::definitions::Definitions;
use super::error;
use crate::program::expression::core::{
    CoreCallTarget, CoreInstruction, CoreInstructionKind, CoreProgram, CoreValueMetadata,
    FunctionRegistry,
};
use crate::program::expression::{ExpressionError, Stage, ValueType};

mod aggregate;
mod binding;
mod block;
mod structural;
mod temporal_attachment;
#[cfg(test)]
mod tests;
pub(super) use block::argument as block_argument;
pub(super) use block::parameters as block_parameters;

pub(super) fn instruction(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
    registry: &FunctionRegistry,
    parameters: &[ValueType],
    parameter_stages: &[Stage],
    captures: &[ValueType],
) -> Result<(), ExpressionError> {
    let expected = match &instruction.kind {
        CoreInstructionKind::Literal(_) => CoreValueMetadata::constant(),
        CoreInstructionKind::Input(id) => id
            .index()
            .and_then(|index| program.inputs.get(index))
            .filter(|input| input.id == *id)
            .ok_or_else(|| error("unknown Core input metadata", instruction.span.clone()))?
            .metadata(),
        CoreInstructionKind::Parameter(index) => binding::parameter(
            *index,
            parameters,
            parameter_stages,
            instruction.span.clone(),
        )?,
        CoreInstructionKind::Capture(index) => {
            binding::capture(*index, captures, instruction.span.clone())?
        }
        CoreInstructionKind::LocalInit { .. }
        | CoreInstructionKind::LocalSet { .. }
        | CoreInstructionKind::LocalGet { .. } => {
            super::local::instruction_metadata(instruction, program, definitions)?
        }
        CoreInstructionKind::Closure {
            definition,
            captures,
        } => {
            let definition = definition
                .index()
                .and_then(|index| program.closure_definitions.get(index))
                .filter(|value| value.id == *definition)
                .ok_or_else(|| error("unknown closure definition", instruction.span.clone()))?;
            let captures = captures
                .iter()
                .map(|id| definitions.metadata(*id).clone())
                .collect::<Vec<_>>();
            CoreValueMetadata::closure(&definition.summary, &captures)
        }
        CoreInstructionKind::Invoke { callee, arguments } => {
            let arguments = arguments
                .iter()
                .map(|id| definitions.metadata(*id).clone())
                .collect::<Vec<_>>();
            let result = program.value_type(instruction.type_id).ok_or_else(|| {
                error(
                    "Invoke result must have a value type",
                    instruction.span.clone(),
                )
            })?;
            CoreValueMetadata::invoke(definitions.metadata(*callee), &arguments, result)
                .ok_or_else(|| {
                    error(
                        "Invoke callee lacks callable metadata",
                        instruction.span.clone(),
                    )
                })?
        }
        CoreInstructionKind::Call {
            target: CoreCallTarget::User(id),
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(|id| definitions.metadata(*id).clone())
                .collect::<Vec<_>>();
            registry
                .get(*id)
                .ok_or_else(|| error(format!("unknown FunctionId {id}"), instruction.span.clone()))?
                .summary()
                .instantiate(&arguments, &[])
                .ok_or_else(|| {
                    error(
                        "user call leaves an unresolved metadata binding",
                        instruction.span.clone(),
                    )
                })?
        }
        CoreInstructionKind::Collection { operation, .. } => {
            aggregate::expected(*operation, instruction, program, definitions)?
        }
        CoreInstructionKind::StructConstruct { fields, .. } => {
            let fields = fields
                .iter()
                .map(|field| definitions.metadata(*field).clone())
                .collect::<Vec<_>>();
            CoreValueMetadata::structure(&fields)
        }
        CoreInstructionKind::EnumConstruct {
            variant, fields, ..
        } => {
            let fields = fields
                .iter()
                .map(|field| definitions.metadata(*field).clone())
                .collect::<Vec<_>>();
            CoreValueMetadata::enumeration(*variant, &fields)
        }
        CoreInstructionKind::List { elements } => structural::list(elements, definitions),
        CoreInstructionKind::Tuple { elements } => structural::tuple(elements, definitions),
        CoreInstructionKind::StructProject { structure, field } => {
            let result = program.value_type(instruction.type_id).ok_or_else(|| {
                error(
                    "StructProject result must have a value type",
                    instruction.span.clone(),
                )
            })?;
            definitions
                .metadata(*structure)
                .struct_field(*field, result)
                .ok_or_else(|| {
                    error(
                        "StructProject lacks a closed field metadata contract",
                        instruction.span.clone(),
                    )
                })?
        }
        CoreInstructionKind::DomainConstruct { .. } | CoreInstructionKind::GraphEmit { .. } => {
            return Err(error(
                "domain metadata requires the versioned operation verifier",
                instruction.span.clone(),
            ));
        }
        CoreInstructionKind::TemporalAttach {
            owner,
            selectors,
            animation,
            ..
        } => temporal_attachment::expected(*owner, selectors, *animation, definitions),
        CoreInstructionKind::TemporalCompose { operands, .. } => {
            CoreValueMetadata::temporal_compose(operands.iter().map(|id| definitions.metadata(*id)))
        }
        CoreInstructionKind::TemporalProject { value, .. } => {
            CoreValueMetadata::temporal_project(definitions.metadata(*value))
        }
        CoreInstructionKind::Range { start, end, step } => CoreValueMetadata::range(
            definitions.metadata(*start),
            definitions.metadata(*end),
            step.map(|id| definitions.metadata(id)),
        ),
        CoreInstructionKind::MapBegin { .. } => CoreValueMetadata::constant(),
        CoreInstructionKind::MapKey { builder, key, .. } => {
            structural::map_key(*builder, *key, definitions)
        }
        CoreInstructionKind::MapValue { pending, value } => {
            structural::map_value(*pending, *value, definitions)
        }
        CoreInstructionKind::MapFinish { builder } => definitions.metadata(*builder).clone(),
        _ => definitions.combined_metadata(instruction.kind.operands()),
    };
    if instruction.metadata != expected {
        return Err(error(
            "instruction metadata does not match its operands",
            instruction.span.clone(),
        ));
    }
    if expected.shape_stage() > Stage::Build {
        return Err(error(
            "value shape cannot depend on Temporal-stage data",
            instruction.span.clone(),
        ));
    }
    Ok(())
}
