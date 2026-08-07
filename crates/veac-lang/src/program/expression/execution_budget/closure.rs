use std::ops::Range;

use super::{execution_limit_error, resource, ExecutionBudget, ResourceDelta};
use crate::program::expression::ExpressionError;

pub(crate) const LOGICAL_CLOSURE_VALUE_BYTES: usize = 64;
pub(crate) const LOGICAL_CLOSURE_BASE_BYTES: usize = 64;
pub(crate) const LOGICAL_CLOSURE_CAPTURE_SLOT_BYTES: usize = 64;

impl ExecutionBudget {
    pub(in crate::program::expression) fn reserve_closure(
        &self,
        capture_count: usize,
        span: Range<usize>,
    ) -> Result<usize, ExpressionError> {
        let resource = resource::Resource::EvaluatorStorageBytes;
        let logical = capture_count
            .checked_mul(LOGICAL_CLOSURE_CAPTURE_SLOT_BYTES)
            .and_then(|bytes| bytes.checked_add(LOGICAL_CLOSURE_BASE_BYTES))
            .ok_or_else(|| {
                execution_limit_error(
                    self.counters[resource.index()].limit,
                    resource.label(),
                    &span,
                )
            })?;
        self.reserve(
            ResourceDelta {
                value_bytes: LOGICAL_CLOSURE_VALUE_BYTES,
                evaluator_storage_bytes: logical,
                ..ResourceDelta::default()
            },
            span,
        )?;
        Ok(logical)
    }
}
