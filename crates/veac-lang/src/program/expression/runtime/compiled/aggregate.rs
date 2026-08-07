use super::iterable::Iterable;
use super::slot::{public, RuntimeValue};
use super::value::contract;
use super::Evaluator;
use crate::program::expression::core::{
    CoreInstruction, CoreProgram, ValueId, VerifiedCoreProgram,
};
use crate::program::expression::{
    CollectionOperation, ExpressionError, Value, ValueType, ValueTypeKind,
};

impl Evaluator<'_> {
    pub(super) fn aggregate_instruction(
        &mut self,
        instruction: &CoreInstruction,
        program: &VerifiedCoreProgram,
        values: &[Option<RuntimeValue>],
        call_depth: usize,
    ) -> Result<Value, ExpressionError> {
        let crate::program::expression::CoreInstructionKind::Collection {
            operation,
            iterable,
            initial,
            callable,
        } = &instruction.kind
        else {
            unreachable!("aggregate evaluator receives a Collection instruction")
        };
        self.aggregate(
            *operation,
            *iterable,
            *initial,
            *callable,
            instruction,
            program.core(),
            values,
            call_depth,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn aggregate(
        &mut self,
        operation: CollectionOperation,
        iterable_id: ValueId,
        initial_id: Option<ValueId>,
        callable_id: ValueId,
        instruction: &CoreInstruction,
        program: &CoreProgram,
        values: &[Option<RuntimeValue>],
        call_depth: usize,
    ) -> Result<Value, ExpressionError> {
        let mut iterable = Iterable::new(
            public(values, iterable_id, instruction)?,
            instruction.span.clone(),
        )?;
        let callable = public(values, callable_id, instruction)?;
        let initial = initial_id
            .map(|id| public(values, id, instruction))
            .transpose()?;
        let count = self.execution.reserve_aggregate(
            operation,
            iterable.is_map(),
            iterable.count(),
            instruction.span.clone(),
        )?;
        match operation {
            CollectionOperation::Map => self.map_values(
                &mut iterable,
                callable,
                count,
                result_element(program, instruction)?,
                instruction,
                call_depth,
            ),
            CollectionOperation::Filter => self.filter_values(
                &mut iterable,
                callable,
                count,
                result_element(program, instruction)?,
                instruction,
                call_depth,
            ),
            CollectionOperation::Fold => self.fold_values(
                &mut iterable,
                callable,
                initial
                    .ok_or_else(|| contract("fold initial value is unavailable", instruction))?,
                instruction,
                call_depth,
            ),
        }
    }

    fn map_values(
        &mut self,
        iterable: &mut Iterable,
        callable: Value,
        count: usize,
        element_type: ValueType,
        instruction: &CoreInstruction,
        call_depth: usize,
    ) -> Result<Value, ExpressionError> {
        let mut output = Vec::with_capacity(count);
        while let Some(value) = iterable.next(instruction.span.clone())? {
            output.push(self.invoke(callable.clone(), vec![value], instruction, call_depth)?);
        }
        Value::list(element_type, output)
            .map_err(|_| contract("verified map output construction failed", instruction))
    }

    fn filter_values(
        &mut self,
        iterable: &mut Iterable,
        callable: Value,
        count: usize,
        element_type: ValueType,
        instruction: &CoreInstruction,
        call_depth: usize,
    ) -> Result<Value, ExpressionError> {
        let mut output = Vec::with_capacity(count);
        while let Some(value) = iterable.next(instruction.span.clone())? {
            let keep = self.invoke(
                callable.clone(),
                vec![value.clone()],
                instruction,
                call_depth,
            )?;
            let Value::Bool(keep) = keep else {
                return Err(contract(
                    "verified filter predicate returned non-bool",
                    instruction,
                ));
            };
            if keep {
                output.push(value);
            }
        }
        Value::list(element_type, output)
            .map_err(|_| contract("verified filter output construction failed", instruction))
    }

    fn fold_values(
        &mut self,
        iterable: &mut Iterable,
        callable: Value,
        mut accumulator: Value,
        instruction: &CoreInstruction,
        call_depth: usize,
    ) -> Result<Value, ExpressionError> {
        while let Some(value) = iterable.next(instruction.span.clone())? {
            accumulator = self.invoke(
                callable.clone(),
                vec![accumulator, value],
                instruction,
                call_depth,
            )?;
        }
        Ok(accumulator)
    }
}

fn result_element(
    program: &CoreProgram,
    instruction: &CoreInstruction,
) -> Result<ValueType, ExpressionError> {
    let result = program
        .value_type(instruction.type_id)
        .ok_or_else(|| contract("aggregate result type is unavailable", instruction))?;
    let ValueTypeKind::List(element) = result.kind() else {
        return Err(contract(
            "aggregate result is not a verified list",
            instruction,
        ));
    };
    Ok(element.clone())
}
