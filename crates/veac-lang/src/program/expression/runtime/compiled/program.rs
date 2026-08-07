use super::control::Flow;
use super::slot::RuntimeValue;
use super::Evaluator;
use crate::program::expression::core::{CompiledFunction, VerifiedCoreProgram};
use crate::program::expression::{ExecutionFrame, ExpressionError, Value};

impl Evaluator<'_> {
    pub(super) fn program(
        &mut self,
        program: &VerifiedCoreProgram,
        parameters: &[Value],
        captures: &[Value],
        call_depth: usize,
        current_function: Option<&CompiledFunction>,
        frame: Option<ExecutionFrame>,
    ) -> Result<Value, ExpressionError> {
        let pushed = frame.is_some();
        if let Some(frame) = frame {
            self.frames.push(frame);
        }
        let result = self.run_program(program, parameters, captures, call_depth, current_function);
        if pushed {
            self.frames.pop();
        }
        if result.is_err() {
            self.domain.taint();
        }
        result
    }

    fn run_program(
        &mut self,
        program: &VerifiedCoreProgram,
        parameters: &[Value],
        captures: &[Value],
        call_depth: usize,
        current_function: Option<&CompiledFunction>,
    ) -> Result<Value, ExpressionError> {
        let core = program.core();
        let slots = core
            .value_count()
            .checked_add(core.inputs.len())
            .and_then(|value| value.checked_add(core.local_slots.len()))
            .ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_EXECUTION_LIMIT",
                    "Core evaluator slot count overflows",
                    super::value::program_span(core),
                )
            })?;
        self.execution
            .reserve_evaluator_slots(slots, super::value::program_span(core))?;
        self.validate_arguments(parameters, captures, program)?;
        let inputs = self.bind_inputs(program, current_function)?;
        let mut values = std::iter::repeat_with(|| None)
            .take(core.value_count())
            .collect::<Vec<Option<RuntimeValue>>>();
        let mut locals = vec![None; core.local_slots.len()];
        let mut block = core.entry;
        loop {
            let current = &core.blocks[block.index().expect("verified block ID")];
            for instruction in &current.instructions {
                let value = self.instruction(
                    instruction,
                    program,
                    &mut values,
                    &mut locals,
                    &inputs,
                    parameters,
                    captures,
                    call_depth,
                    current_function,
                )?;
                values[instruction.id.index().expect("verified value ID")] = Some(value);
            }
            match self.terminator(current, &mut values, program, call_depth, current_function)? {
                Flow::Return(value) => return Ok(value),
                Flow::Continue(target) => block = target,
            }
        }
    }
}
