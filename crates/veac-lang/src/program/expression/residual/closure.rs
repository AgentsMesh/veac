use std::collections::BTreeMap;
use std::sync::Arc;

use super::evaluator::Evaluator;
use super::{ResidualBuildBindings, ResidualRuntimeValue};
use super::{ResidualLedger, ResidualizationError, ResidualizationRequest, ResidualizedExpression};
use crate::program::expression::core::CoreInstructionKind;
use crate::program::expression::{
    ClosureValue, CompiledExpression, CoreTemporalInputIdentity, Effect, ExactNumber, InputId,
    PrimitiveType, Value, ValueType,
};

pub(in crate::program) fn residualize_closure_with_ledger(
    closure: &ClosureValue,
    inputs: &[Option<CoreTemporalInputIdentity>],
    request: ResidualizationRequest,
    ledger: ResidualLedger<'_>,
) -> Result<ResidualizedExpression, ResidualizationError> {
    if closure.summary().effect() != Effect::Pure {
        return Err(error("animation closure must be Pure"));
    }
    if inputs.len() != closure.definition().parameter_types().len() {
        return Err(error("animation clock signature is incomplete"));
    }
    let expression = CompiledExpression::new(
        closure.definition().body().clone(),
        Arc::clone(closure.registry()),
    );
    let bindings = ResidualBuildBindings::new();
    Evaluator::new(&expression, &bindings, request, ledger).run_closure(closure, inputs)
}

impl Evaluator<'_> {
    fn run_closure(
        mut self,
        closure: &ClosureValue,
        inputs: &[Option<CoreTemporalInputIdentity>],
    ) -> Result<ResidualizedExpression, ResidualizationError> {
        let used = used_parameters(self.core());
        let parameters = closure
            .definition()
            .parameter_types()
            .iter()
            .enumerate()
            .map(|(index, value_type)| {
                let Some(span) = used.get(&index) else {
                    return concrete_zero(value_type);
                };
                let identity = inputs[index]
                    .as_ref()
                    .ok_or_else(|| error("animation uses an unavailable owner clock"))?;
                if identity.value_type() != *value_type {
                    return Err(error(
                        "animation clock type does not match its closure parameter",
                    ));
                }
                self.builder
                    .synthetic_input(InputId::new(index as u32), identity, span.clone())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let captures = closure
            .captures()
            .iter()
            .cloned()
            .map(ResidualRuntimeValue::Concrete)
            .collect();
        let value = self.call_program(self.core().clone(), parameters, captures)?;
        self.builder.finish(value, self.request)
    }
}

fn used_parameters(
    core: &crate::program::expression::CoreProgram,
) -> BTreeMap<usize, std::ops::Range<usize>> {
    core.blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .filter_map(|instruction| match instruction.kind() {
            CoreInstructionKind::Parameter(index) => Some((*index, instruction.span())),
            _ => None,
        })
        .collect()
}

fn concrete_zero(value_type: &ValueType) -> Result<ResidualRuntimeValue, ResidualizationError> {
    let zero = ExactNumber::integer(0);
    let value = match value_type.as_primitive() {
        Some(PrimitiveType::Time) => Value::Time(zero),
        Some(PrimitiveType::Integer) => Value::Integer(0),
        Some(PrimitiveType::Scalar) => Value::Scalar(zero),
        _ => return Err(error("unused animation clock has an unsupported type")),
    };
    Ok(ResidualRuntimeValue::Concrete(value))
}

fn error(message: impl Into<String>) -> ResidualizationError {
    ResidualizationError::new("RESIDUAL_ANIMATION_CLOSURE", message, 0..0)
}
