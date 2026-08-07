use std::ops::Range;

use super::{ExecutionBudget, ResourceDelta};
use crate::program::expression::{ExpressionError, Value};

impl ExecutionBudget {
    pub(in crate::program::expression) fn charge_value(
        &self,
        value: &Value,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.reserve_value_bytes(value.evaluated_bytes(), span)
    }

    pub(in crate::program::expression) fn reserve_value_bytes(
        &self,
        bytes: usize,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.reserve(
            ResourceDelta {
                value_bytes: bytes,
                ..ResourceDelta::default()
            },
            span,
        )
    }
}
