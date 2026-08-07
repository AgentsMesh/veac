use std::ops::Range;

use super::{
    execution_limit_error, resource, ExecutionBudget, ResourceDelta, LOGICAL_COLLECTION_BASE_BYTES,
    LOGICAL_COLLECTION_HANDLE_BYTES,
};
use crate::program::expression::{CollectionOperation, ExpressionError};

impl ExecutionBudget {
    pub(in crate::program::expression) fn reserve_aggregate(
        &self,
        operation: CollectionOperation,
        map_input: bool,
        count: u64,
        span: Range<usize>,
    ) -> Result<usize, ExpressionError> {
        let Ok(count) = usize::try_from(count) else {
            return Err(self.overflow_iterations(&span));
        };
        self.preview(resource::Resource::Iterations, count, &span)?;
        let output = matches!(
            operation,
            CollectionOperation::Map | CollectionOperation::Filter
        );
        let output_elements = if output { count } else { 0 };
        let tuple_elements = if map_input {
            checked_mul(
                self,
                count,
                2,
                resource::Resource::CollectionElements,
                &span,
            )?
        } else {
            0
        };
        let Some(collection_elements) = output_elements.checked_add(tuple_elements) else {
            return Err(self.overflow(resource::Resource::CollectionElements, &span));
        };
        self.preview(
            resource::Resource::CollectionElements,
            collection_elements,
            &span,
        )?;
        let output_bytes = if output {
            collection_bytes(self, count, 1, true, &span)?
        } else {
            0
        };
        let tuple_bytes = if map_input {
            collection_bytes(self, count, 2, false, &span)?
        } else {
            0
        };
        let Some(collection_bytes) = output_bytes.checked_add(tuple_bytes) else {
            return Err(self.overflow(resource::Resource::CollectionBytes, &span));
        };
        self.reserve(
            ResourceDelta {
                iterations: count,
                collection_elements,
                collection_bytes,
                ..ResourceDelta::default()
            },
            span,
        )?;
        Ok(count)
    }

    fn overflow_iterations(&self, span: &Range<usize>) -> ExpressionError {
        self.overflow(resource::Resource::Iterations, span)
    }

    fn overflow(&self, resource: resource::Resource, span: &Range<usize>) -> ExpressionError {
        execution_limit_error(
            self.counters[resource.index()].limit,
            resource.label(),
            span,
        )
    }

    fn preview(
        &self,
        resource: resource::Resource,
        amount: usize,
        span: &Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.counters[resource.index()]
            .preview(amount, resource.label(), span)
            .map(|_| ())
    }
}

fn collection_bytes(
    budget: &ExecutionBudget,
    count: usize,
    handles: usize,
    single_base: bool,
    span: &Range<usize>,
) -> Result<usize, ExpressionError> {
    let resource = resource::Resource::CollectionBytes;
    let handles = checked_mul(budget, count, handles, resource, span)?;
    let handles = checked_mul(
        budget,
        handles,
        LOGICAL_COLLECTION_HANDLE_BYTES,
        resource,
        span,
    )?;
    let bases = if single_base {
        LOGICAL_COLLECTION_BASE_BYTES
    } else {
        checked_mul(budget, count, LOGICAL_COLLECTION_BASE_BYTES, resource, span)?
    };
    match handles.checked_add(bases) {
        Some(value) => Ok(value),
        None => Err(budget.overflow(resource, span)),
    }
}

fn checked_mul(
    budget: &ExecutionBudget,
    left: usize,
    right: usize,
    resource: resource::Resource,
    span: &Range<usize>,
) -> Result<usize, ExpressionError> {
    match left.checked_mul(right) {
        Some(value) => Ok(value),
        None => Err(budget.overflow(resource, span)),
    }
}

#[cfg(test)]
#[path = "aggregate/overflow_tests.rs"]
mod overflow_tests;
