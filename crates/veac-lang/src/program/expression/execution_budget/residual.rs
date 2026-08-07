use std::ops::Range;

use super::{ExecutionBudget, ResourceDelta};
use crate::program::expression::ExpressionError;

pub(super) const LOGICAL_RESIDUAL_NODE_BYTES: usize = 64;

#[derive(Debug, Clone, Copy)]
pub(in crate::program) struct ResidualLedger<'a> {
    execution: &'a ExecutionBudget,
}

impl ExecutionBudget {
    pub(in crate::program) const fn residual_ledger(&self) -> ResidualLedger<'_> {
        ResidualLedger { execution: self }
    }
}

impl<'a> ResidualLedger<'a> {
    pub(in crate::program::expression) const fn execution(self) -> &'a ExecutionBudget {
        self.execution
    }

    pub(in crate::program::expression) fn transaction<T, E>(
        self,
        operation: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        self.execution.transaction(operation)
    }

    pub(in crate::program::expression) fn reserve_step(
        &self,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.execution.reserve(
            ResourceDelta {
                fuel: 1,
                ..ResourceDelta::default()
            },
            span,
        )
    }

    pub(in crate::program::expression) fn reserve_node(
        &self,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.execution.reserve(
            ResourceDelta {
                residual_nodes: 1,
                residual_bytes: LOGICAL_RESIDUAL_NODE_BYTES,
                ..ResourceDelta::default()
            },
            span,
        )
    }

    pub(in crate::program::expression) fn reserve_value(
        &self,
        bytes: usize,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.execution.reserve(
            ResourceDelta {
                residual_bytes: bytes,
                ..ResourceDelta::default()
            },
            span,
        )
    }
}
