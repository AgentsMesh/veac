use std::cmp::Ordering;
use std::ops::Range;

use crate::program::expression::{ExpressionError, Value};

pub(super) fn call(
    name: &str,
    arguments: Vec<Value>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    match name {
        "min" => extremum(name, arguments, span, Ordering::Less),
        "max" => extremum(name, arguments, span, Ordering::Greater),
        "clamp" => clamp(arguments, span),
        "identifier" => identifier(name, arguments, span),
        _ => Err(ExpressionError::new(
            "EXPRESSION_UNKNOWN_FUNCTION",
            format!("unknown function `{name}`"),
            span,
        )),
    }
}

fn identifier(
    name: &str,
    mut arguments: Vec<Value>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    require_arity(name, &arguments, 1, span.clone())?;
    let Value::Text(value) = arguments.remove(0) else {
        return Err(ExpressionError::new(
            "EXPRESSION_TYPE",
            format!("{name} expects one text argument"),
            span,
        ));
    };
    if !crate::name::is_name(&value) {
        return Err(ExpressionError::new(
            "EXPRESSION_IDENTIFIER",
            format!("identifier must be {}", crate::name::NAME_CONTRACT),
            span,
        ));
    }
    Ok(Value::Identifier(value))
}

fn extremum(
    name: &str,
    arguments: Vec<Value>,
    span: Range<usize>,
    wanted: Ordering,
) -> Result<Value, ExpressionError> {
    require_arity(name, &arguments, 2, span.clone())?;
    comparable(&arguments, span.clone())?;
    let comparison = arguments[0]
        .numeric()
        .expect("comparable values are numeric")
        .compare(
            arguments[1]
                .numeric()
                .expect("comparable values are numeric"),
        );
    Ok(if comparison == wanted {
        arguments[0].clone()
    } else {
        arguments[1].clone()
    })
}

fn clamp(arguments: Vec<Value>, span: Range<usize>) -> Result<Value, ExpressionError> {
    require_arity("clamp", &arguments, 3, span.clone())?;
    comparable(&arguments, span.clone())?;
    let value = arguments[0].numeric().expect("validated numeric value");
    let minimum = arguments[1].numeric().expect("validated numeric value");
    let maximum = arguments[2].numeric().expect("validated numeric value");
    if minimum.compare(maximum) == Ordering::Greater {
        return Err(ExpressionError::new(
            "EXPRESSION_CLAMP_RANGE",
            "clamp minimum must not exceed its maximum",
            span,
        ));
    }
    Ok(if value.compare(minimum) == Ordering::Less {
        arguments[1].clone()
    } else if value.compare(maximum) == Ordering::Greater {
        arguments[2].clone()
    } else {
        arguments[0].clone()
    })
}

fn comparable(arguments: &[Value], span: Range<usize>) -> Result<(), ExpressionError> {
    let Some(first) = arguments.first() else {
        return Ok(());
    };
    if !first.kind().is_numeric() || arguments.iter().any(|value| value.kind() != first.kind()) {
        return Err(ExpressionError::new(
            "EXPRESSION_TYPE",
            "function arguments must have one numeric kind",
            span,
        ));
    }
    Ok(())
}

fn require_arity(
    name: &str,
    arguments: &[Value],
    expected: usize,
    span: Range<usize>,
) -> Result<(), ExpressionError> {
    if arguments.len() != expected {
        return Err(ExpressionError::new(
            "EXPRESSION_ARITY",
            format!(
                "{name} expects {expected} arguments, found {}",
                arguments.len()
            ),
            span,
        ));
    }
    Ok(())
}
