use std::cmp::Ordering;
use std::ops::Range;

use crate::program::expression::{BuiltinFunction, ExpressionError, Value};

pub(in crate::program::expression) fn call_builtin(
    function: BuiltinFunction,
    arguments: Vec<Value>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    let name = function.as_str();
    match function {
        BuiltinFunction::Min => extremum(name, arguments, span, Ordering::Less),
        BuiltinFunction::Max => extremum(name, arguments, span, Ordering::Greater),
        BuiltinFunction::Clamp => clamp(arguments, span),
        BuiltinFunction::Identifier => identifier(name, arguments, span),
        function if function.is_curve() => Err(ExpressionError::new(
            "EXPRESSION_TEMPORAL_CURVE_CONTEXT",
            format!("{} requires a Temporal input", function.as_str()),
            span,
        )),
        _ => unreachable!("every builtin has runtime semantics"),
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
    let kind = first.primitive_kind();
    if !kind.is_some_and(|value| value.is_numeric())
        || arguments.iter().any(|value| value.primitive_kind() != kind)
    {
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
