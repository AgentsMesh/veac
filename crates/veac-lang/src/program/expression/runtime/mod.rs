mod function;
mod operation;

use super::ast::{Expression, ExpressionKind, UnaryOperator};
use super::{
    ExpressionError, Value, ValueLookup, MAX_EXPRESSION_DEPTH, MAX_EXPRESSION_NODES,
    MAX_TEXT_VALUE_BYTES,
};

pub(super) fn evaluate(
    expression: &Expression,
    environment: &dyn ValueLookup,
) -> Result<Value, ExpressionError> {
    Evaluator {
        environment,
        nodes: 0,
    }
    .visit(expression, 1)
}

struct Evaluator<'a> {
    environment: &'a dyn ValueLookup,
    nodes: usize,
}

impl Evaluator<'_> {
    fn visit(&mut self, expression: &Expression, depth: usize) -> Result<Value, ExpressionError> {
        if depth > MAX_EXPRESSION_DEPTH {
            return Err(ExpressionError::new(
                "EXPRESSION_DEPTH_LIMIT",
                format!("expression exceeds the {MAX_EXPRESSION_DEPTH} evaluation depth limit"),
                expression.span.clone(),
            ));
        }
        self.nodes += 1;
        if self.nodes > MAX_EXPRESSION_NODES {
            return Err(ExpressionError::new(
                "EXPRESSION_NODE_LIMIT",
                format!("expression exceeds the {MAX_EXPRESSION_NODES} node limit"),
                expression.span.clone(),
            ));
        }
        let value = match &expression.kind {
            ExpressionKind::Literal(value) => Ok(value.clone()),
            ExpressionKind::Symbol(name) => {
                self.environment.value(name).cloned().ok_or_else(|| {
                    ExpressionError::new(
                        "EXPRESSION_UNKNOWN_SYMBOL",
                        format!("unknown symbol `{name}`"),
                        expression.span.clone(),
                    )
                })
            }
            ExpressionKind::Unary { operator, operand } => {
                let value = self.visit(operand, depth + 1)?;
                unary(*operator, value, expression)
            }
            ExpressionKind::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.visit(left, depth + 1)?;
                let right = self.visit(right, depth + 1)?;
                operation::apply(*operator, left, right, expression.span.clone())
            }
            ExpressionKind::Call {
                function,
                arguments,
            } => {
                let values = arguments
                    .iter()
                    .map(|value| self.visit(value, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?;
                function::call(function, values, expression.span.clone())
            }
        }?;
        validate_value(value, expression)
    }
}

fn validate_value(value: Value, expression: &Expression) -> Result<Value, ExpressionError> {
    if matches!(&value, Value::Text(text) if text.len() > MAX_TEXT_VALUE_BYTES) {
        return Err(ExpressionError::new(
            "EXPRESSION_TEXT_LIMIT",
            format!("text value exceeds the {MAX_TEXT_VALUE_BYTES} byte limit"),
            expression.span.clone(),
        ));
    }
    Ok(value)
}

fn unary(
    operator: UnaryOperator,
    value: Value,
    expression: &Expression,
) -> Result<Value, ExpressionError> {
    let Some(number) = value.numeric() else {
        return Err(ExpressionError::new(
            "EXPRESSION_TYPE",
            format!(
                "unary sign requires a numeric value, found {}",
                value.kind()
            ),
            expression.span.clone(),
        ));
    };
    match operator {
        UnaryOperator::Positive => Ok(value),
        UnaryOperator::Negative => number
            .checked_neg()
            .and_then(|number| Value::from_numeric(value.kind(), number))
            .ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_OVERFLOW",
                    "unary negation exceeds the exact arithmetic range",
                    expression.span.clone(),
                )
            }),
    }
}
