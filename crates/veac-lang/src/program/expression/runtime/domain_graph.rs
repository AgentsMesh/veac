use std::ops::Range;
use std::sync::Arc;

use crate::program::DomainOperationRegistry;

use super::super::value::domain::DomainGraphScope;
use super::super::{
    DomainOrigin, DomainValue, ExecutionBudget, ExpressionError, ProgramIdentity, Value,
};

mod accounting;
mod argument;
mod freeze;
mod operation;
mod provenance;
mod state;
mod temporal_attachment;
mod traversal;
mod validation;

use state::{ArenaState, NodeId};
pub(in crate::program) use traversal::{FrozenEntity, FrozenTemporalAttachment};

pub(in crate::program::expression) struct DomainGraphTransaction<'a> {
    registry: &'a DomainOperationRegistry,
    execution: &'a ExecutionBudget,
    scope: DomainGraphScope,
    state: ArenaState,
    failed: bool,
    identity: Option<ProgramIdentity>,
}

pub(in crate::program) struct FrozenDomainGraph {
    scope: DomainGraphScope,
    state: ArenaState,
    root: Arc<DomainValue>,
    root_node: NodeId,
    identity: Option<ProgramIdentity>,
}

impl std::fmt::Debug for FrozenDomainGraph {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrozenDomainGraph")
            .field("records", &self.state.records.len())
            .field("root", &self.root)
            .finish()
    }
}

impl<'a> DomainGraphTransaction<'a> {
    pub(in crate::program::expression) fn new(
        registry: &'a DomainOperationRegistry,
        execution: &'a ExecutionBudget,
    ) -> Self {
        Self {
            registry,
            execution,
            scope: DomainGraphScope::fresh(),
            state: ArenaState::default(),
            failed: false,
            identity: None,
        }
    }

    pub(in crate::program::expression) fn evaluate(
        &mut self,
        opcode: u16,
        operands: Vec<Value>,
        span: Range<usize>,
    ) -> Result<Value, ExpressionError> {
        self.ensure_active(&span)?;
        let result = operation::evaluate(self, opcode, operands, span, None);
        if result.is_err() {
            self.taint();
        }
        result
    }

    pub(in crate::program::expression) fn evaluate_authored(
        &mut self,
        opcode: u16,
        operands: Vec<Value>,
        span: Range<usize>,
        origin: Option<DomainOrigin>,
    ) -> Result<Value, ExpressionError> {
        self.ensure_active(&span)?;
        let result = operation::evaluate(self, opcode, operands, span, origin);
        if result.is_err() {
            self.taint();
        }
        result
    }

    pub(in crate::program::expression) fn taint(&mut self) {
        self.failed = true;
    }

    pub(in crate::program::expression) fn set_program_identity(
        &mut self,
        identity: ProgramIdentity,
        span: Range<usize>,
    ) -> Result<(), ExpressionError> {
        self.ensure_active(&span)?;
        let bytes = identity.logical_bytes();
        if self.identity.is_some() {
            self.failed = true;
            return Err(error(
                "DOMAIN_PROGRAM_IDENTITY",
                "a graph transaction accepts one executable program identity",
                span,
            ));
        }
        if let Err(error) = self.execution.reserve_domain_graph(0, bytes, span) {
            self.failed = true;
            return Err(error);
        }
        self.state.logical_bytes += bytes;
        self.identity = Some(identity);
        Ok(())
    }

    pub(in crate::program::expression) fn host_context(
        &mut self,
        span: Range<usize>,
    ) -> Result<Value, ExpressionError> {
        self.ensure_active(&span)?;
        let bytes = accounting::LOGICAL_DOMAIN_RECORD_BYTES;
        if let Err(error) = self.execution.reserve_domain_graph(0, bytes, span) {
            self.failed = true;
            return Err(error);
        }
        Ok(self.state.insert_host_context(&self.scope, bytes))
    }

    pub(in crate::program::expression) fn freeze(
        self,
        root: &Value,
        span: Range<usize>,
    ) -> Result<FrozenDomainGraph, ExpressionError> {
        if self.failed {
            return Err(error(
                "DOMAIN_TRANSACTION_FAILED",
                "a failed graph transaction cannot be published",
                span,
            ));
        }
        let (root, root_node) = freeze::validate(&self, root, span)?;
        Ok(FrozenDomainGraph {
            scope: self.scope,
            state: self.state,
            root,
            root_node,
            identity: self.identity,
        })
    }

    pub(in crate::program::expression) fn record_count(&self) -> usize {
        self.state.records.len()
    }

    fn ensure_active(&self, span: &Range<usize>) -> Result<(), ExpressionError> {
        (!self.failed).then_some(()).ok_or_else(|| {
            error(
                "DOMAIN_TRANSACTION_FAILED",
                "a failed graph transaction cannot continue",
                span.clone(),
            )
        })
    }
}

pub(super) fn error(
    code: &'static str,
    message: impl Into<String>,
    span: Range<usize>,
) -> ExpressionError {
    ExpressionError::new(code, message, span)
}

#[cfg(test)]
#[path = "domain_graph/tests.rs"]
mod tests;
