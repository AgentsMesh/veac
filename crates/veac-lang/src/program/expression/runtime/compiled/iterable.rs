use crate::program::expression::{ExpressionError, Value};

pub(super) struct Iterable {
    source: Value,
    index: u64,
}

impl Iterable {
    pub(super) fn new(
        source: Value,
        span: std::ops::Range<usize>,
    ) -> Result<Self, ExpressionError> {
        if !matches!(source, Value::List(_) | Value::Range(_) | Value::Map(_)) {
            return Err(contract(
                "collection input is not a verified iterable value",
                span,
            ));
        }
        Ok(Self { source, index: 0 })
    }

    pub(super) fn count(&self) -> u64 {
        match &self.source {
            Value::List(value) => value.values().len() as u64,
            Value::Range(value) => value.count(),
            Value::Map(value) => value.entries().len() as u64,
            _ => unreachable!("Iterable constructor validates its source"),
        }
    }

    pub(super) fn is_map(&self) -> bool {
        matches!(self.source, Value::Map(_))
    }

    pub(super) fn next(
        &mut self,
        span: std::ops::Range<usize>,
    ) -> Result<Option<Value>, ExpressionError> {
        if self.index >= self.count() {
            return Ok(None);
        }
        let index = self.index;
        self.index += 1;
        let value = match &self.source {
            Value::List(value) => value.values()[index as usize].clone(),
            Value::Range(value) => {
                let item = i128::from(value.start()) + i128::from(value.step()) * i128::from(index);
                Value::Integer(
                    i64::try_from(item).map_err(|_| {
                        contract("verified range iteration overflowed", span.clone())
                    })?,
                )
            }
            Value::Map(value) => {
                let entry = &value.entries()[index as usize];
                Value::tuple(vec![entry.key().clone(), entry.value().clone()])
                    .map_err(|_| contract("verified map entry tuple construction failed", span))?
            }
            _ => unreachable!("Iterable constructor validates its source"),
        };
        Ok(Some(value))
    }
}

fn contract(message: &str, span: std::ops::Range<usize>) -> ExpressionError {
    ExpressionError::new("EXPRESSION_RUNTIME_CONTRACT", message, span)
}

#[cfg(test)]
#[path = "iterable/tests.rs"]
mod tests;
