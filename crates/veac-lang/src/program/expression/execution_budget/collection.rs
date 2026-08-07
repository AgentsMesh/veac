use std::ops::Range;

use super::{
    execution_limit_error, resource, ExecutionBudget, ResourceDelta, LOGICAL_COLLECTION_BASE_BYTES,
    LOGICAL_COLLECTION_HANDLE_BYTES,
};
use crate::program::expression::ExpressionError;

impl ExecutionBudget {
    pub(in crate::program::expression) fn reserve_sequence_collection(
        &self,
        count: usize,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.reserve_collection(count, 1, span)
    }

    pub(in crate::program::expression) fn reserve_map_collection(
        &self,
        count: usize,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.reserve_collection(count, 2, span)
    }

    fn reserve_collection(
        &self,
        count: usize,
        handles_per_element: usize,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        let resource = resource::Resource::CollectionBytes;
        let bytes = count
            .checked_mul(handles_per_element)
            .and_then(|handles| handles.checked_mul(LOGICAL_COLLECTION_HANDLE_BYTES))
            .and_then(|bytes| bytes.checked_add(LOGICAL_COLLECTION_BASE_BYTES))
            .ok_or_else(|| {
                execution_limit_error(
                    self.counters[resource.index()].limit,
                    resource.label(),
                    &span,
                )
            })?;
        self.reserve(
            ResourceDelta {
                collection_elements: count,
                collection_bytes: bytes,
                ..ResourceDelta::default()
            },
            span,
        )
    }
}
