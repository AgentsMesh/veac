use std::ops::Range;

use super::{ResidualizationError, ResidualizationLimits};
use crate::program::expression::ResidualLedger;

#[derive(Debug)]
pub(super) struct Budget<'a> {
    limits: ResidualizationLimits,
    steps: usize,
    nodes: usize,
    value_bytes: usize,
    ledger: ResidualLedger<'a>,
}

impl<'a> Budget<'a> {
    pub(super) const fn new(limits: ResidualizationLimits, ledger: ResidualLedger<'a>) -> Self {
        Self {
            limits,
            steps: 0,
            nodes: 0,
            value_bytes: 0,
            ledger,
        }
    }

    pub(super) fn step(&mut self, span: Range<usize>) -> Result<(), ResidualizationError> {
        charge(
            &mut self.steps,
            1,
            self.limits.max_steps,
            "RESIDUAL_STEP_LIMIT",
            "residualization step limit exceeded",
            span.clone(),
        )?;
        self.ledger
            .reserve_step(span)
            .map_err(ResidualizationError::expression)
    }

    pub(super) fn node(&mut self, span: Range<usize>) -> Result<(), ResidualizationError> {
        charge(
            &mut self.nodes,
            1,
            self.limits.max_nodes,
            "RESIDUAL_NODE_LIMIT",
            "residual node limit exceeded",
            span.clone(),
        )?;
        self.ledger
            .reserve_node(span)
            .map_err(ResidualizationError::expression)
    }

    pub(super) fn value(
        &mut self,
        amount: usize,
        span: Range<usize>,
    ) -> Result<(), ResidualizationError> {
        charge(
            &mut self.value_bytes,
            amount,
            self.limits.max_value_bytes,
            "RESIDUAL_VALUE_LIMIT",
            "residual value byte limit exceeded",
            span.clone(),
        )?;
        self.ledger
            .reserve_value(amount, span)
            .map_err(ResidualizationError::expression)
    }
}

fn charge(
    current: &mut usize,
    amount: usize,
    limit: usize,
    code: &'static str,
    message: &'static str,
    span: Range<usize>,
) -> Result<(), ResidualizationError> {
    let next = current.checked_add(amount);
    match next {
        Some(next) if next <= limit => {
            *current = next;
            Ok(())
        }
        _ => Err(ResidualizationError::new(code, message, span)),
    }
}
