use super::error;
use crate::program::expression::residual::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{Value, ValueTypeKind};

pub(in crate::program::expression::residual) struct ResidualIterable {
    source: ResidualRuntimeValue,
    index: usize,
}

impl ResidualIterable {
    pub(in crate::program::expression::residual) fn new(
        source: ResidualRuntimeValue,
        span: std::ops::Range<usize>,
    ) -> Result<Self, ResidualizationError> {
        let valid = matches!(
            source,
            ResidualRuntimeValue::Concrete(Value::List(_) | Value::Range(_) | Value::Map(_))
        ) || matches!(&source, ResidualRuntimeValue::Sequence { value_type, .. } if matches!(value_type.kind(), ValueTypeKind::List(_)));
        valid.then_some(Self { source, index: 0 }).ok_or_else(|| {
            error(
                "RESIDUAL_CORE_CONTRACT",
                "for-each input is not iterable",
                span,
            )
        })
    }

    pub(in crate::program::expression::residual) fn count(&self) -> usize {
        match &self.source {
            ResidualRuntimeValue::Concrete(Value::List(value)) => value.values().len(),
            ResidualRuntimeValue::Concrete(Value::Range(value)) => value.count() as usize,
            ResidualRuntimeValue::Concrete(Value::Map(value)) => value.entries().len(),
            ResidualRuntimeValue::Sequence { values, .. } => values.len(),
            _ => unreachable!("residual iterable is validated"),
        }
    }

    pub(in crate::program::expression::residual) fn is_map(&self) -> bool {
        matches!(self.source, ResidualRuntimeValue::Concrete(Value::Map(_)))
    }

    pub(in crate::program::expression::residual) fn next(
        &mut self,
        span: std::ops::Range<usize>,
    ) -> Result<Option<ResidualRuntimeValue>, ResidualizationError> {
        if self.index >= self.count() {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        let value = match &self.source {
            ResidualRuntimeValue::Concrete(Value::List(value)) => {
                ResidualRuntimeValue::Concrete(value.values()[index].clone())
            }
            ResidualRuntimeValue::Concrete(Value::Range(value)) => {
                let item = i128::from(value.start()) + i128::from(value.step()) * index as i128;
                ResidualRuntimeValue::Concrete(Value::Integer(i64::try_from(item).map_err(
                    |_| {
                        error(
                            "RESIDUAL_CORE_CONTRACT",
                            "for-each range overflowed",
                            span.clone(),
                        )
                    },
                )?))
            }
            ResidualRuntimeValue::Concrete(Value::Map(value)) => {
                let entry = &value.entries()[index];
                let pair = Value::tuple(vec![entry.key().clone(), entry.value().clone()]).map_err(
                    |_| {
                        error(
                            "RESIDUAL_CORE_CONTRACT",
                            "for-each map entry is invalid",
                            span,
                        )
                    },
                )?;
                ResidualRuntimeValue::Concrete(pair)
            }
            ResidualRuntimeValue::Sequence { values, .. } => values[index].clone(),
            _ => unreachable!("residual iterable is validated"),
        };
        Ok(Some(value))
    }
}
