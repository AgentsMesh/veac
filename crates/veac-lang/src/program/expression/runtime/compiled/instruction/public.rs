use super::super::slot::{public, RuntimeValue};
use super::super::value::contract;
use super::super::Evaluator;
use crate::program::expression::core::{
    CompiledFunction, CoreInstruction, CoreInstructionKind, VerifiedCoreProgram,
};
use crate::program::expression::{ExpressionError, Value};

impl Evaluator<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn public_instruction(
        &mut self,
        instruction: &CoreInstruction,
        program: &VerifiedCoreProgram,
        values: &[Option<RuntimeValue>],
        inputs: &[Value],
        parameters: &[Value],
        captures: &[Value],
        call_depth: usize,
        current_function: Option<&CompiledFunction>,
    ) -> Result<Value, ExpressionError> {
        Ok(match &instruction.kind {
            CoreInstructionKind::Literal(value) => value.clone(),
            CoreInstructionKind::Input(id) => inputs
                .get(id.index().unwrap_or(usize::MAX))
                .cloned()
                .ok_or_else(|| contract("compiled input index is out of bounds", instruction))?,
            CoreInstructionKind::Parameter(index) => {
                parameters.get(*index).cloned().ok_or_else(|| {
                    contract("compiled parameter index is out of bounds", instruction)
                })?
            }
            CoreInstructionKind::Capture(index) => captures
                .get(*index)
                .cloned()
                .ok_or_else(|| contract("compiled capture index is out of bounds", instruction))?,
            CoreInstructionKind::Closure {
                definition,
                captures,
            } => self.closure(*definition, captures, instruction, program, values)?,
            CoreInstructionKind::Invoke { callee, arguments } => {
                let callee = public(values, *callee, instruction)?;
                let arguments = arguments
                    .iter()
                    .map(|id| public(values, *id, instruction))
                    .collect::<Result<Vec<_>, _>>()?;
                self.invoke(callee, arguments, instruction, call_depth)?
            }
            CoreInstructionKind::Unary { operator, operand } => super::super::super::unary(
                *operator,
                public(values, *operand, instruction)?,
                instruction.span.clone(),
            )?,
            CoreInstructionKind::Arithmetic {
                operator,
                left,
                right,
            } => super::super::super::operation::apply(
                self.execution,
                *operator,
                public(values, *left, instruction)?,
                public(values, *right, instruction)?,
                instruction.span.clone(),
            )?,
            CoreInstructionKind::Compare {
                operator,
                left,
                right,
            } => super::super::super::compare::ordering(
                *operator,
                public(values, *left, instruction)?,
                public(values, *right, instruction)?,
                instruction.span.clone(),
            )?,
            CoreInstructionKind::Equal {
                operator,
                left,
                right,
            } => super::super::super::compare::equality(
                *operator,
                public(values, *left, instruction)?,
                public(values, *right, instruction)?,
                instruction.span.clone(),
            )?,
            CoreInstructionKind::Call { target, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|id| public(values, *id, instruction))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call(
                    *target,
                    arguments,
                    instruction,
                    call_depth,
                    current_function,
                )?
            }
            CoreInstructionKind::Collection { .. } => {
                self.aggregate_instruction(instruction, program, values, call_depth)?
            }
            CoreInstructionKind::List { .. }
            | CoreInstructionKind::Tuple { .. }
            | CoreInstructionKind::Range { .. }
            | CoreInstructionKind::MapBegin { .. }
            | CoreInstructionKind::MapKey { .. }
            | CoreInstructionKind::MapValue { .. }
            | CoreInstructionKind::MapFinish { .. } => {
                return Err(contract("Core instruction is not public", instruction));
            }
            CoreInstructionKind::StructConstruct { .. }
            | CoreInstructionKind::StructProject { .. }
            | CoreInstructionKind::EnumConstruct { .. }
            | CoreInstructionKind::DomainConstruct { .. }
            | CoreInstructionKind::GraphEmit { .. }
            | CoreInstructionKind::TemporalAttach { .. }
            | CoreInstructionKind::LocalInit { .. }
            | CoreInstructionKind::LocalSet { .. }
            | CoreInstructionKind::LocalGet { .. } => {
                return Err(contract("Core instruction routing failed", instruction));
            }
            CoreInstructionKind::TemporalCompose { .. }
            | CoreInstructionKind::TemporalProject { .. } => {
                return Err(ExpressionError::new(
                    "EXPRESSION_TEMPORAL_RESIDUALIZATION_REQUIRED",
                    "Temporal composite Core instruction requires residualization",
                    instruction.span(),
                ));
            }
        })
    }
}
