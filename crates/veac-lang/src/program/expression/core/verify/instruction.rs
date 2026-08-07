use super::control::ControlFlow;
use super::definitions::Definitions;
use super::{
    aggregate, collection, domain, error, local, metadata, nominal_value, range, temporal,
    type_table, types,
};
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreProgram, FunctionRegistry, ValueId,
};
use crate::program::expression::{ExpressionError, Stage, ValueType};

mod call;
mod callable;
mod temporal_attachment;

#[allow(clippy::too_many_arguments)]
pub(super) fn verify(
    instruction: &CoreInstruction,
    block: usize,
    position: usize,
    program: &CoreProgram,
    definitions: &Definitions,
    control: &ControlFlow,
    registry: &FunctionRegistry,
    domain_registry: &crate::program::DomainOperationRegistry,
    nominal_types: &crate::program::TypeRegistry,
    parameters: &[ValueType],
    parameter_stages: &[Stage],
    captures: &[ValueType],
) -> Result<(), ExpressionError> {
    let span = instruction.span.clone();
    for operand in instruction.kind.operands() {
        definitions.verify_use(operand, block, position, control, span.clone())?;
    }
    let verify_metadata = || {
        metadata::instruction(
            instruction,
            program,
            definitions,
            registry,
            parameters,
            parameter_stages,
            captures,
        )
    };
    let expected = match &instruction.kind {
        CoreInstructionKind::Literal(value) => {
            value
                .validate_nominal_registry(nominal_types)
                .map_err(|failure| error(failure.message(), span.clone()))?;
            value.value_type()
        }
        CoreInstructionKind::Input(id) => {
            let id = definitions
                .input_type_id(*id)
                .ok_or_else(|| error(format!("unknown input ID {}", id.value()), span.clone()))?;
            type_table::value(program, id, span.clone())?.clone()
        }
        CoreInstructionKind::Parameter(index) => parameters
            .get(*index)
            .cloned()
            .ok_or_else(|| error(format!("unknown parameter index {index}"), span.clone()))?,
        CoreInstructionKind::Capture(index) => captures
            .get(*index)
            .cloned()
            .ok_or_else(|| error(format!("unknown capture index {index}"), span.clone()))?,
        CoreInstructionKind::LocalInit { .. }
        | CoreInstructionKind::LocalSet { .. }
        | CoreInstructionKind::LocalGet { .. } => {
            local::instruction_type(instruction, program, definitions)?
        }
        CoreInstructionKind::Closure {
            definition,
            captures,
        } => callable::closure(*definition, captures, program, definitions, span.clone())?,
        CoreInstructionKind::Invoke { callee, arguments } => {
            callable::invoke(*callee, arguments, program, definitions, span.clone())?
        }
        CoreInstructionKind::Unary { operator, operand } => types::unary(
            *operator,
            value_type(program, definitions, *operand, span.clone())?,
            span.clone(),
        )?,
        CoreInstructionKind::Arithmetic {
            operator,
            left,
            right,
        } => types::arithmetic(
            *operator,
            value_type(program, definitions, *left, span.clone())?,
            value_type(program, definitions, *right, span.clone())?,
            span.clone(),
        )?,
        CoreInstructionKind::Compare { left, right, .. } => types::comparison(
            value_type(program, definitions, *left, span.clone())?,
            value_type(program, definitions, *right, span.clone())?,
            span.clone(),
        )?,
        CoreInstructionKind::Equal { left, right, .. } => types::equality(
            value_type(program, definitions, *left, span.clone())?,
            value_type(program, definitions, *right, span.clone())?,
            nominal_types,
            span.clone(),
        )?,
        CoreInstructionKind::Call { target, arguments } => call::verify(
            *target,
            arguments,
            program,
            definitions,
            registry,
            span.clone(),
        )?,
        CoreInstructionKind::Collection { .. } => {
            aggregate::verify(instruction, program, definitions, nominal_types)?;
            verify_metadata()?;
            return Ok(());
        }
        CoreInstructionKind::StructConstruct { type_id, fields } => nominal_value::structure(
            *type_id,
            fields,
            program,
            definitions,
            nominal_types,
            span.clone(),
        )?,
        CoreInstructionKind::StructProject { structure, field } => nominal_value::project(
            *structure,
            *field,
            program,
            definitions,
            nominal_types,
            span.clone(),
        )?,
        CoreInstructionKind::EnumConstruct {
            type_id,
            variant,
            fields,
        } => nominal_value::enumeration(
            *type_id,
            *variant,
            fields,
            program,
            definitions,
            nominal_types,
            span.clone(),
        )?,
        CoreInstructionKind::DomainConstruct { .. } | CoreInstructionKind::GraphEmit { .. } => {
            domain::instruction(instruction, program, definitions, domain_registry)?;
            return Ok(());
        }
        CoreInstructionKind::TemporalAttach { .. } => {
            temporal_attachment::verify(instruction, program, definitions)?;
            verify_metadata()?;
            return Ok(());
        }
        CoreInstructionKind::TemporalCompose { .. }
        | CoreInstructionKind::TemporalProject { .. } => {
            temporal::instruction(instruction, program, definitions)?
        }
        CoreInstructionKind::List { .. }
        | CoreInstructionKind::Tuple { .. }
        | CoreInstructionKind::MapBegin { .. }
        | CoreInstructionKind::MapKey { .. }
        | CoreInstructionKind::MapValue { .. }
        | CoreInstructionKind::MapFinish { .. } => {
            collection::verify(instruction, program, definitions)?;
            verify_metadata()?;
            return Ok(());
        }
        CoreInstructionKind::Range { .. } => {
            range::verify(instruction, program, definitions)?;
            verify_metadata()?;
            return Ok(());
        }
    };
    verify_metadata()?;
    let actual = type_table::value(program, instruction.type_id, span.clone())?;
    (actual == &expected).then_some(()).ok_or_else(|| {
        error(
            format!("instruction declares {actual}, expected {expected}"),
            span,
        )
    })
}

pub(super) fn value_type<'a>(
    program: &'a CoreProgram,
    definitions: &Definitions,
    id: ValueId,
    span: Range<usize>,
) -> Result<&'a ValueType, ExpressionError> {
    type_table::value(program, definitions.type_id(id), span)
}
use std::ops::Range;
