use super::builder::Builder;
use super::{
    ResidualBuildBindings, ResidualRuntimeValue, ResidualizationError, ResidualizationRequest,
    ResidualizedExpression,
};
use crate::program::expression::{
    CompiledExpression, CoreInputIdentity, CoreProgram, ExecutionBudget, ResidualLedger, Value,
};
use crate::program::DomainOperationRegistry;
use std::collections::BTreeMap;

pub(super) struct Evaluator<'a> {
    pub(super) expression: &'a CompiledExpression,
    pub(super) bindings: &'a ResidualBuildBindings,
    pub(super) builder: Builder<'a>,
    pub(super) concrete_budget: &'a ExecutionBudget,
    pub(super) domain: DomainOperationRegistry,
    pub(super) current_core: CoreProgram,
    pub(super) parameters: Vec<ResidualRuntimeValue>,
    pub(super) captures: Vec<ResidualRuntimeValue>,
    pub(super) map_states:
        BTreeMap<(usize, crate::program::expression::ValueId), super::map_state::MapState>,
    pub(super) closures: BTreeMap<
        (usize, crate::program::expression::ValueId),
        super::closure_state::ResidualClosure,
    >,
    pub(super) call_depth: usize,
    pub(super) request: ResidualizationRequest,
}

impl<'a> Evaluator<'a> {
    pub(super) fn new(
        expression: &'a CompiledExpression,
        bindings: &'a ResidualBuildBindings,
        request: ResidualizationRequest,
        ledger: ResidualLedger<'a>,
    ) -> Self {
        let current_core = expression.core().clone();
        Self {
            expression,
            bindings,
            builder: Builder::new(&request, ledger),
            concrete_budget: ledger.execution(),
            domain: DomainOperationRegistry::standard(),
            current_core,
            parameters: Vec::new(),
            captures: Vec::new(),
            map_states: BTreeMap::new(),
            closures: BTreeMap::new(),
            call_depth: 0,
            request,
        }
    }

    pub(super) fn run(mut self) -> Result<ResidualizedExpression, ResidualizationError> {
        self.validate_bindings()?;
        let slots = vec![None; self.core().value_count()];
        let value = self.block(self.core().entry(), slots)?;
        self.builder.finish(value, self.request)
    }

    pub(super) fn core(&self) -> &CoreProgram {
        &self.current_core
    }

    pub(super) fn value(
        &self,
        slots: &[Option<ResidualRuntimeValue>],
        id: crate::program::expression::ValueId,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        id.index()
            .and_then(|index| slots.get(index))
            .and_then(Clone::clone)
            .ok_or_else(|| {
                ResidualizationError::new(
                    "RESIDUAL_CORE_CONTRACT",
                    format!("Core value {} is unavailable", id.value()),
                    span,
                )
            })
    }

    fn validate_bindings(&self) -> Result<(), ResidualizationError> {
        for input in self.core().inputs() {
            let CoreInputIdentity::Build(id) = input.identity() else {
                continue;
            };
            let value = self.bindings.get(id).ok_or_else(|| {
                ResidualizationError::new(
                    "RESIDUAL_BUILD_INPUT_MISSING",
                    format!("missing Build input `{}` ({id})", input.name()),
                    input.span(),
                )
            })?;
            let expected = self
                .core()
                .value_type(input.type_id())
                .expect("verified Core input has a value type");
            if &value.value_type() != expected {
                return Err(ResidualizationError::new(
                    "RESIDUAL_BUILD_INPUT_TYPE",
                    format!("Build input `{}` expects {expected}", input.name()),
                    input.span(),
                ));
            }
            value
                .validate_nominal_registry(self.expression.verified().nominal_types())
                .map_err(|error| {
                    ResidualizationError::new(
                        "RESIDUAL_BUILD_INPUT_CONTRACT",
                        error.message(),
                        input.span(),
                    )
                })?;
        }
        Ok(())
    }

    pub(super) fn concrete(&self, value: Value) -> ResidualRuntimeValue {
        ResidualRuntimeValue::Concrete(value)
    }
}
