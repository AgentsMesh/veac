use super::slot::{public, RuntimeValue};
use super::value::contract;
use super::Evaluator;
use crate::program::expression::core::{CoreInstruction, CoreProgram, ValueId};
use crate::program::expression::{ExpressionError, Value};

impl Evaluator<'_> {
    pub(super) fn range(
        &self,
        start: ValueId,
        end: ValueId,
        step: Option<ValueId>,
        instruction: &CoreInstruction,
        program: &CoreProgram,
        values: &[Option<RuntimeValue>],
    ) -> Result<RuntimeValue, ExpressionError> {
        let start = integer(values, start, instruction)?;
        let end = integer(values, end, instruction)?;
        let step_value = step
            .map(|id| integer(values, id, instruction))
            .transpose()?
            .unwrap_or(1);
        if step_value == 0 {
            let span = step
                .and_then(|id| program.value_span(id))
                .unwrap_or_else(|| instruction.span.clone());
            return Err(ExpressionError::new(
                "EXPRESSION_RANGE_STEP",
                "range step cannot be zero",
                span,
            ));
        }
        Value::range(start, end, step_value)
            .map(RuntimeValue::Public)
            .map_err(|_| contract("Core range construction failed", instruction))
    }
}

fn integer(
    values: &[Option<RuntimeValue>],
    id: ValueId,
    instruction: &CoreInstruction,
) -> Result<i64, ExpressionError> {
    match public(values, id, instruction)? {
        Value::Integer(value) => Ok(value),
        _ => Err(contract("Core range operand is not int", instruction)),
    }
}
