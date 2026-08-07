use super::super::super::evaluator::Evaluator;
use super::super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{BuiltinFunction, ComparisonOperator, Value};
use veac_ir::{TemporalCompareOperation, TemporalNodeKind, TemporalType};

mod curve;

impl Evaluator<'_> {
    pub(super) fn builtin(
        &mut self,
        function: BuiltinFunction,
        arguments: Vec<ResidualRuntimeValue>,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        if let Some(concrete) = concrete(&arguments) {
            return crate::program::expression::runtime::residual_builtin(function, concrete, span)
                .map(ResidualRuntimeValue::Concrete)
                .map_err(ResidualizationError::expression);
        }
        match function {
            BuiltinFunction::Min => self.extremum(arguments, TemporalCompareOperation::Less, span),
            BuiltinFunction::Max => {
                self.extremum(arguments, TemporalCompareOperation::Greater, span)
            }
            BuiltinFunction::Clamp => self.clamp(arguments, span),
            BuiltinFunction::Identifier => Err(ResidualizationError::new(
                "RESIDUAL_IDENTIFIER_TEMPORAL",
                "identifier construction cannot depend on Temporal-stage text",
                span,
            )),
            function if function.is_curve() => self.curve(function, arguments, span),
            _ => unreachable!("every builtin has residual semantics"),
        }
    }

    fn extremum(
        &mut self,
        arguments: Vec<ResidualRuntimeValue>,
        operation: TemporalCompareOperation,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let [left, right]: [ResidualRuntimeValue; 2] = arguments.try_into().map_err(|_| {
            ResidualizationError::new(
                "RESIDUAL_CORE_CONTRACT",
                "extremum arity mismatch",
                span.clone(),
            )
        })?;
        let condition = self.compare_value(operation, left.clone(), right.clone(), span.clone())?;
        self.select(condition, left, right, span)
    }

    fn clamp(
        &mut self,
        arguments: Vec<ResidualRuntimeValue>,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let [value, minimum, maximum]: [ResidualRuntimeValue; 3] =
            arguments.try_into().map_err(|_| {
                ResidualizationError::new(
                    "RESIDUAL_CORE_CONTRACT",
                    "clamp arity mismatch",
                    span.clone(),
                )
            })?;
        validate_range(&minimum, &maximum, span.clone())?;
        let lower = self.extremum(
            vec![value, minimum],
            TemporalCompareOperation::Greater,
            span.clone(),
        )?;
        self.extremum(vec![lower, maximum], TemporalCompareOperation::Less, span)
    }

    fn compare_value(
        &mut self,
        operation: TemporalCompareOperation,
        left: ResidualRuntimeValue,
        right: ResidualRuntimeValue,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        if let Some([left, right]) = concrete_pair(&left, &right) {
            let operator = match operation {
                TemporalCompareOperation::Less => ComparisonOperator::Less,
                TemporalCompareOperation::Greater => ComparisonOperator::Greater,
                _ => unreachable!("extremum uses an ordering operation"),
            };
            return crate::program::expression::runtime::residual_ordering(
                operator, left, right, span,
            )
            .map(ResidualRuntimeValue::Concrete)
            .map_err(ResidualizationError::expression);
        }
        let left = self.builder.as_node(left, span.clone())?;
        let right = self.builder.as_node(right, span.clone())?;
        self.builder
            .node(
                TemporalType::Boolean,
                TemporalNodeKind::Compare {
                    operation,
                    left: left.node_id(),
                    right: right.node_id(),
                },
                span,
            )
            .map(ResidualRuntimeValue::Residual)
    }
}

fn concrete(values: &[ResidualRuntimeValue]) -> Option<Vec<Value>> {
    values
        .iter()
        .map(|value| match value {
            ResidualRuntimeValue::Concrete(value) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

fn concrete_pair(left: &ResidualRuntimeValue, right: &ResidualRuntimeValue) -> Option<[Value; 2]> {
    let ResidualRuntimeValue::Concrete(left) = left else {
        return None;
    };
    let ResidualRuntimeValue::Concrete(right) = right else {
        return None;
    };
    Some([left.clone(), right.clone()])
}

fn validate_range(
    minimum: &ResidualRuntimeValue,
    maximum: &ResidualRuntimeValue,
    span: std::ops::Range<usize>,
) -> Result<(), ResidualizationError> {
    let Some([left, right]) = concrete_pair(minimum, maximum) else {
        return Err(ResidualizationError::new(
            "RESIDUAL_CLAMP_DYNAMIC_RANGE",
            "temporal clamp requires Build-stage minimum and maximum bounds",
            span,
        ));
    };
    let value = crate::program::expression::runtime::residual_ordering(
        ComparisonOperator::Greater,
        left,
        right,
        span.clone(),
    )
    .map_err(ResidualizationError::expression)?;
    match value {
        Value::Bool(false) => Ok(()),
        Value::Bool(true) => Err(ResidualizationError::new(
            "RESIDUAL_CLAMP_RANGE",
            "clamp minimum must not exceed its maximum",
            span,
        )),
        _ => unreachable!("verified comparison returns bool"),
    }
}
