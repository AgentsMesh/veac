use std::ops::Range;

use super::{
    execution_limit_error, resource, ExecutionBudget, ResourceDelta, LOGICAL_NOMINAL_BASE_BYTES,
    LOGICAL_NOMINAL_FIELD_HANDLE_BYTES,
};
use crate::program::expression::ExpressionError;

impl ExecutionBudget {
    pub(in crate::program::expression) fn reserve_nominal(
        &self,
        fields: usize,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        let resource = resource::Resource::CollectionBytes;
        let bytes = fields
            .checked_mul(LOGICAL_NOMINAL_FIELD_HANDLE_BYTES)
            .and_then(|bytes| bytes.checked_add(LOGICAL_NOMINAL_BASE_BYTES))
            .ok_or_else(|| {
                execution_limit_error(
                    self.counters[resource.index()].limit,
                    resource.label(),
                    &span,
                )
            })?;
        self.reserve(
            ResourceDelta {
                collection_elements: fields,
                collection_bytes: bytes,
                ..ResourceDelta::default()
            },
            span,
        )
    }
}
