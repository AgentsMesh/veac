use super::super::super::super::evaluator::Evaluator;
use super::super::super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::BuiltinFunction;
use veac_ir::{Interpolation, TemporalNodeKind};

mod keys;
mod validation;

impl Evaluator<'_> {
    pub(super) fn curve(
        &mut self,
        function: BuiltinFunction,
        arguments: Vec<ResidualRuntimeValue>,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let ResidualRuntimeValue::Residual(input) = &arguments[0] else {
            return Err(error(
                "RESIDUAL_CURVE_CONTEXT",
                "curve sampling requires a Temporal input",
                span,
            ));
        };
        let interpolation = interpolation(function, &arguments[2..], span.clone())?;
        let keys = keys::curve_keys(&arguments[1], interpolation, span.clone())?;
        validation::keys(input.value_type(), &keys, span.clone())?;
        for key in &keys {
            self.builder
                .budget
                .value(key.value.logical_bytes(), span.clone())?;
        }
        let value_type = keys[0].value.value_type();
        self.builder
            .node(
                value_type,
                TemporalNodeKind::CurveSample {
                    input: input.node_id(),
                    keys,
                },
                span,
            )
            .map(ResidualRuntimeValue::Residual)
    }
}

fn interpolation(
    function: BuiltinFunction,
    parameters: &[ResidualRuntimeValue],
    span: std::ops::Range<usize>,
) -> Result<Interpolation, ResidualizationError> {
    let numbers = parameters
        .iter()
        .map(|value| keys::scalar(value, span.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let value = match function {
        BuiltinFunction::SampleCurveHold => Interpolation::Hold,
        BuiltinFunction::SampleCurveLinear => Interpolation::Linear,
        BuiltinFunction::SampleCurveEaseIn => Interpolation::EaseIn,
        BuiltinFunction::SampleCurveEaseOut => Interpolation::EaseOut,
        BuiltinFunction::SampleCurveEaseInOut => Interpolation::EaseInOut,
        BuiltinFunction::SampleCurveSpring => Interpolation::Spring {
            frequency: numbers[0],
            decay: numbers[1],
            initial_velocity: numbers[2],
        },
        BuiltinFunction::SampleCurveCubicBezier => Interpolation::CubicBezier {
            x1: numbers[0],
            y1: numbers[1],
            x2: numbers[2],
            y2: numbers[3],
        },
        _ => unreachable!("curve dispatch only passes curve builtins"),
    };
    validation::interpolation(&value)
        .then_some(value)
        .ok_or_else(|| {
            error(
                "RESIDUAL_CURVE_INTERPOLATION",
                "curve interpolation parameters are outside the canonical contract",
                span,
            )
        })
}

pub(super) fn error(
    code: &'static str,
    message: &'static str,
    span: std::ops::Range<usize>,
) -> ResidualizationError {
    ResidualizationError::new(code, message, span)
}
