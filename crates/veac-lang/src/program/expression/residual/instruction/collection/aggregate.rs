use super::super::super::evaluator::Evaluator;
use super::super::super::for_each::{iterable::ResidualIterable, pack};
use super::super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{CollectionOperation, CoreInstruction, Value, ValueId};

impl Evaluator<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::program::expression::residual) fn aggregate(
        &mut self,
        operation: CollectionOperation,
        iterable: ValueId,
        initial: Option<ValueId>,
        callable: ValueId,
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let span = instruction.span();
        let source = self.value(slots, iterable, span.clone())?;
        let mut iterable = ResidualIterable::new(source, span.clone())?;
        self.concrete_budget
            .reserve_aggregate(
                operation,
                iterable.is_map(),
                iterable.count() as u64,
                span.clone(),
            )
            .map_err(ResidualizationError::expression)?;
        let closure = self
            .closures
            .get(&(self.call_depth, callable))
            .cloned()
            .ok_or_else(|| contract("collection callback is unavailable", instruction))?;
        let definition = closure
            .definition
            .index()
            .and_then(|index| self.core().closure_definitions().get(index))
            .cloned()
            .ok_or_else(|| contract("collection callback body is unavailable", instruction))?;
        let initial = initial
            .map(|id| self.value(slots, id, span.clone()))
            .transpose()?;
        match operation {
            CollectionOperation::Map => {
                let mut values = Vec::with_capacity(iterable.count());
                while let Some(value) = iterable.next(span.clone())? {
                    values.push(self.invoke_collection(
                        &definition,
                        &closure.captures,
                        vec![value],
                    )?);
                }
                let value_type = self.result_type(instruction, "map")?;
                pack(value_type, values, span)
            }
            CollectionOperation::Filter => {
                let mut values = Vec::with_capacity(iterable.count());
                while let Some(value) = iterable.next(span.clone())? {
                    let keep = self.invoke_collection(
                        &definition,
                        &closure.captures,
                        vec![value.clone()],
                    )?;
                    match keep {
                        ResidualRuntimeValue::Concrete(Value::Bool(true)) => values.push(value),
                        ResidualRuntimeValue::Concrete(Value::Bool(false)) => {}
                        _ => {
                            return Err(contract(
                                "filter predicate is not Build-stage bool",
                                instruction,
                            ))
                        }
                    }
                }
                let value_type = self.result_type(instruction, "filter")?;
                pack(value_type, values, span)
            }
            CollectionOperation::Fold => {
                let mut output = initial
                    .ok_or_else(|| contract("fold initial value is unavailable", instruction))?;
                while let Some(value) = iterable.next(span.clone())? {
                    output = self.invoke_collection(
                        &definition,
                        &closure.captures,
                        vec![output, value],
                    )?;
                }
                Ok(output)
            }
        }
    }

    fn invoke_collection(
        &mut self,
        definition: &crate::program::expression::CoreClosureDefinition,
        captures: &[ResidualRuntimeValue],
        parameters: Vec<ResidualRuntimeValue>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        self.call_program(definition.body().clone(), parameters, captures.to_vec())
    }

    fn result_type(
        &self,
        instruction: &CoreInstruction,
        operation: &str,
    ) -> Result<crate::program::expression::ValueType, ResidualizationError> {
        self.core()
            .value_type(instruction.type_id())
            .cloned()
            .ok_or_else(|| {
                contract(
                    &format!("{operation} result type is unavailable"),
                    instruction,
                )
            })
    }
}

fn contract(message: &str, instruction: &CoreInstruction) -> ResidualizationError {
    ResidualizationError::new("RESIDUAL_CORE_CONTRACT", message, instruction.span())
}
