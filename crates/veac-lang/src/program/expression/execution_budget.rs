use std::cell::Cell;
use std::ops::Range;

use super::ExpressionError;

mod admission;
mod aggregate;
mod closure;
mod collection;
mod delta;
mod limits;
mod nominal;
mod residual;
mod resource;
mod transaction;
mod value;

pub(crate) use closure::LOGICAL_CLOSURE_VALUE_BYTES;
#[cfg(test)]
pub(crate) use closure::{LOGICAL_CLOSURE_BASE_BYTES, LOGICAL_CLOSURE_CAPTURE_SLOT_BYTES};
pub(crate) use delta::ResourceDelta;
pub(in crate::program) use residual::ResidualLedger;
pub(crate) type ExecutionLimits = limits::ResourceLimits;
use resource::{RESOURCES, RESOURCE_COUNT};

pub(crate) const MAX_EXECUTION_FUEL: usize = 1_048_576;
pub(crate) const MAX_EVALUATED_VALUE_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_EXECUTION_ITERATIONS: usize = 1_048_576;
pub(crate) const MAX_COLLECTION_ELEMENTS: usize = 1_048_576;
pub(crate) const MAX_COLLECTION_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_EMITTED_ENTITIES: usize = 65_536;
pub(crate) const MAX_EMITTED_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_RESIDUAL_NODES: usize = 262_144;
pub(crate) const MAX_RESIDUAL_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_EVALUATOR_STORAGE_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const LOGICAL_CORE_SLOT_BYTES: usize = 64;
pub(crate) const LOGICAL_COLLECTION_BASE_BYTES: usize = 64;
pub(crate) const LOGICAL_COLLECTION_HANDLE_BYTES: usize = 64;
pub(crate) const LOGICAL_NOMINAL_BASE_BYTES: usize = 64;
pub(crate) const LOGICAL_NOMINAL_FIELD_HANDLE_BYTES: usize = 64;
pub(crate) const LOGICAL_RANGE_VALUE_BYTES: usize = 4 * 8;

#[derive(Debug)]
pub(crate) struct ExecutionBudget {
    counters: [Counter; RESOURCE_COUNT],
}

impl ExecutionBudget {
    pub(crate) fn with_limit(limit: usize) -> Self {
        Self::with_limits(limit, MAX_EVALUATED_VALUE_BYTES)
    }

    pub(crate) fn with_limits(fuel: usize, value_bytes: usize) -> Self {
        let limits = ExecutionLimits {
            fuel,
            value_bytes,
            ..ExecutionLimits::default()
        };
        Self::with_resource_limits(limits)
    }

    pub(crate) fn with_resource_limits(limits: ExecutionLimits) -> Self {
        Self {
            counters: RESOURCES.map(|resource| Counter::new(resource.limit(&limits))),
        }
    }

    pub(super) fn charge_node(&self, span: Range<usize>) -> Result<(), ExpressionError> {
        self.reserve(
            ResourceDelta {
                fuel: 1,
                ..ResourceDelta::default()
            },
            span,
        )
    }

    pub(super) fn reserve_evaluator_slots(
        &self,
        count: usize,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        let resource = resource::Resource::EvaluatorStorageBytes;
        let amount = count.checked_mul(LOGICAL_CORE_SLOT_BYTES).ok_or_else(|| {
            execution_limit_error(
                self.counters[resource.index()].limit,
                resource.label(),
                &span,
            )
        })?;
        self.reserve(
            ResourceDelta {
                evaluator_storage_bytes: amount,
                ..ResourceDelta::default()
            },
            span,
        )
    }

    pub(in crate::program::expression) fn reserve_domain_graph(
        &self,
        entities: usize,
        bytes: usize,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.reserve(
            ResourceDelta {
                emitted_entities: entities,
                emitted_bytes: bytes,
                ..ResourceDelta::default()
            },
            span,
        )
    }

    pub(super) fn reserve(
        &self,
        delta: ResourceDelta,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        let mut next = [0; RESOURCE_COUNT];
        for resource in RESOURCES {
            next[resource.index()] = self.counters[resource.index()].preview(
                resource.amount(&delta),
                resource.label(),
                &span,
            )?;
        }
        for resource in RESOURCES {
            self.counters[resource.index()].commit(next[resource.index()]);
        }
        Ok(())
    }
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self::with_limit(MAX_EXECUTION_FUEL)
    }
}

#[derive(Debug)]
struct Counter {
    used: Cell<usize>,
    limit: usize,
}

impl Counter {
    fn new(limit: usize) -> Self {
        Self {
            used: Cell::new(0),
            limit,
        }
    }

    fn preview(
        &self,
        amount: usize,
        dimension: &str,
        span: &Range<usize>,
    ) -> Result<usize, ExpressionError> {
        let next = self.used.get().checked_add(amount);
        match next {
            Some(next) if next <= self.limit => Ok(next),
            _ => Err(execution_limit_error(self.limit, dimension, span)),
        }
    }

    fn commit(&self, next: usize) {
        self.used.set(next);
    }
}

fn execution_limit_error(limit: usize, dimension: &str, span: &Range<usize>) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_EXECUTION_LIMIT",
        format!("program expression execution exceeds the {limit} {dimension} limit"),
        span.clone(),
    )
}

#[cfg(test)]
#[path = "execution_budget/tests.rs"]
mod tests;
