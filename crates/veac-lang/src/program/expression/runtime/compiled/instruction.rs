use super::slot::{verify_contract, RuntimeValue};
use super::value::annotate;
use super::Evaluator;
use crate::program::expression::core::{
    CompiledFunction, CoreInstruction, CoreInstructionKind, VerifiedCoreProgram,
};
use crate::program::expression::{ExpressionError, Value};
mod account;
mod public;

impl Evaluator<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn instruction(
        &mut self,
        instruction: &CoreInstruction,
        program: &VerifiedCoreProgram,
        values: &mut [Option<RuntimeValue>],
        locals: &mut [Option<Value>],
        inputs: &[Value],
        parameters: &[Value],
        captures: &[Value],
        call_depth: usize,
        current_function: Option<&CompiledFunction>,
    ) -> Result<RuntimeValue, ExpressionError> {
        let result = (|| {
            self.enter(instruction.span.clone())?;
            let runtime = match &instruction.kind {
                CoreInstructionKind::List { elements } => {
                    self.sequence(elements, false, instruction, program.core(), values)?
                }
                CoreInstructionKind::Tuple { elements } => {
                    self.sequence(elements, true, instruction, program.core(), values)?
                }
                CoreInstructionKind::Range { start, end, step } => {
                    self.range(*start, *end, *step, instruction, program.core(), values)?
                }
                CoreInstructionKind::MapBegin { entries } => {
                    self.map_begin(*entries, instruction, program.core())?
                }
                CoreInstructionKind::MapKey {
                    builder,
                    key,
                    ordinal,
                } => self.map_key(*builder, *key, *ordinal, instruction, values)?,
                CoreInstructionKind::MapValue { pending, value } => {
                    self.map_value(*pending, *value, instruction, program.core(), values)?
                }
                CoreInstructionKind::MapFinish { builder } => {
                    self.map_finish(*builder, instruction, program.core(), values)?
                }
                CoreInstructionKind::StructConstruct { type_id, fields } => RuntimeValue::Public(
                    self.struct_construct(*type_id, fields, instruction, program, values)?,
                ),
                CoreInstructionKind::StructProject { structure, field } => RuntimeValue::Public(
                    self.struct_project(*structure, *field, instruction, values)?,
                ),
                CoreInstructionKind::EnumConstruct {
                    type_id,
                    variant,
                    fields,
                } => RuntimeValue::Public(self.enum_construct(
                    *type_id,
                    *variant,
                    fields,
                    instruction,
                    program,
                    values,
                )?),
                CoreInstructionKind::DomainConstruct { opcode, operands }
                | CoreInstructionKind::GraphEmit { opcode, operands } => RuntimeValue::Public(
                    self.domain_instruction(*opcode, operands, instruction, values)?,
                ),
                CoreInstructionKind::TemporalAttach {
                    kind,
                    owner,
                    selectors,
                    animation,
                } => RuntimeValue::Public(self.temporal_attachment(
                    *kind,
                    *owner,
                    selectors,
                    *animation,
                    instruction,
                    values,
                )?),
                CoreInstructionKind::LocalInit { .. }
                | CoreInstructionKind::LocalSet { .. }
                | CoreInstructionKind::LocalGet { .. } => {
                    RuntimeValue::Public(self.local(instruction, values, locals)?)
                }
                _ => RuntimeValue::Public(self.public_instruction(
                    instruction,
                    program,
                    values,
                    inputs,
                    parameters,
                    captures,
                    call_depth,
                    current_function,
                )?),
            };
            verify_contract(&runtime, instruction, program.core())?;
            if let RuntimeValue::Public(value) = runtime {
                let value = super::super::validate_value(value, instruction.span.clone())?;
                self.account_public(&value, instruction)?;
                Ok(RuntimeValue::Public(value))
            } else {
                Ok(runtime)
            }
        })();
        annotate(result, current_function)
    }
}
