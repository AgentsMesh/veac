use super::evaluator::Evaluator;
use super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{
    CollectionOperation, CoreForEach, Effect, Value, ValueType, ValueTypeKind,
    MAX_FUNCTION_CALL_DEPTH,
};

pub(super) mod iterable;

use iterable::ResidualIterable;

impl Evaluator<'_> {
    pub(super) fn for_each(
        &mut self,
        loop_value: &CoreForEach,
        mut slots: Vec<Option<ResidualRuntimeValue>>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let span = loop_value.provenance().loop_span().clone();
        if loop_value.effect().summary() != Effect::Pure
            || loop_value.effect().contains_local_mutation()
        {
            return Err(error(
                "RESIDUAL_EFFECT_UNSUPPORTED",
                "residualization only executes Pure for-each bodies",
                span,
            ));
        }
        if self.call_depth >= MAX_FUNCTION_CALL_DEPTH {
            return Err(error(
                "RESIDUAL_CALL_DEPTH_LIMIT",
                format!("function calls exceed the {MAX_FUNCTION_CALL_DEPTH} depth limit"),
                span,
            ));
        }
        let source = self.value(&slots, loop_value.iterable(), span.clone())?;
        let mut iterable = ResidualIterable::new(source, span.clone())?;
        if iterable.count() > loop_value.maximum_count() as usize {
            return Err(error(
                "RESIDUAL_CORE_CONTRACT",
                "for-each input exceeds its verified static bound",
                span,
            ));
        }
        self.concrete_budget
            .reserve_aggregate(
                CollectionOperation::Map,
                iterable.is_map(),
                iterable.count() as u64,
                span.clone(),
            )
            .map_err(ResidualizationError::expression)?;
        let captures = loop_value
            .captures()
            .iter()
            .map(|id| self.value(&slots, *id, span.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let definition = self
            .core()
            .closure_definitions()
            .get(loop_value.body().index().expect("verified body"))
            .cloned()
            .ok_or_else(|| {
                error(
                    "RESIDUAL_CORE_CONTRACT",
                    "for-each body is unavailable",
                    span.clone(),
                )
            })?;
        let target = loop_value.continuation();
        let result_type = self.core().blocks()[target.index().expect("verified continuation")]
            .parameters()[0]
            .type_id();
        let result_type = self
            .core()
            .value_type(result_type)
            .cloned()
            .ok_or_else(|| {
                error(
                    "RESIDUAL_CORE_CONTRACT",
                    "for-each result type is unavailable",
                    span.clone(),
                )
            })?;
        let mut output = Vec::with_capacity(iterable.count());
        let mut index = 0i64;
        while let Some(element) = iterable.next(span.clone())? {
            output.push(self.call_program(
                definition.body().clone(),
                vec![
                    element,
                    ResidualRuntimeValue::Concrete(Value::Integer(index)),
                ],
                captures.clone(),
            )?);
            index += 1;
        }
        let result = pack(result_type, output, span.clone())?;
        let parameter = self.core().blocks()[target.index().expect("verified continuation")]
            .parameters()[0]
            .id();
        slots[parameter.index().expect("verified parameter")] = Some(result);
        self.block(target, slots)
    }
}

pub(super) fn pack(
    value_type: ValueType,
    values: Vec<ResidualRuntimeValue>,
    span: std::ops::Range<usize>,
) -> Result<ResidualRuntimeValue, ResidualizationError> {
    if values
        .iter()
        .all(|value| matches!(value, ResidualRuntimeValue::Concrete(_)))
    {
        let ValueTypeKind::List(element) = value_type.kind() else {
            return Err(error(
                "RESIDUAL_CORE_CONTRACT",
                "for-each result is not a list",
                span,
            ));
        };
        let concrete = values
            .into_iter()
            .map(|value| match value {
                ResidualRuntimeValue::Concrete(value) => value,
                _ => unreachable!(),
            })
            .collect();
        return Value::list(element.clone(), concrete)
            .map(ResidualRuntimeValue::Concrete)
            .map_err(|_| error("RESIDUAL_CORE_CONTRACT", "for-each result is invalid", span));
    }
    Ok(ResidualRuntimeValue::Sequence { value_type, values })
}

fn error(
    code: &'static str,
    message: impl Into<String>,
    span: std::ops::Range<usize>,
) -> ResidualizationError {
    ResidualizationError::new(code, message, span)
}
