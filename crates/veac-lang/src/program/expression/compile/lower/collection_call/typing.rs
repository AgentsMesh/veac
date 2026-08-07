use crate::program::expression::ast::Expression;
use crate::program::expression::hir::TypedNode;
use crate::program::expression::{
    CollectionOperation, ExpressionError, FunctionEffect, ValueType, ValueTypeKind,
};

pub(in crate::program::expression::compile::lower) fn iterable_element(
    value_type: &ValueType,
    expression: &Expression,
) -> Result<ValueType, ExpressionError> {
    match value_type.kind() {
        ValueTypeKind::List(value) | ValueTypeKind::Range(value) => Ok(value.clone()),
        ValueTypeKind::Map { key, value } => {
            ValueType::tuple(vec![key.primitive().into(), value.clone()])
                .map_err(|error| type_error(error.message(), expression))
        }
        _ => Err(type_error(
            format!("expected list, range, or map, found {value_type}"),
            expression,
        )),
    }
}

pub(super) fn callback_result(
    operation: CollectionOperation,
    callback: &TypedNode,
    expected_parameters: &[ValueType],
    expected_result: Option<&ValueType>,
    expression: &Expression,
) -> Result<ValueType, ExpressionError> {
    let ValueTypeKind::Function {
        parameters,
        result,
        effect,
    } = callback.value_type.kind()
    else {
        return Err(type_error(
            format!("expected function value, found {}", callback.value_type),
            expression,
        ));
    };
    if parameters != expected_parameters || expected_result.is_some_and(|value| value != result) {
        return Err(type_error(
            "collection callback signature does not match its input",
            expression,
        ));
    }
    require_effect_contract(operation, effect, expression)?;
    Ok(result.clone())
}

fn require_effect_contract(
    operation: CollectionOperation,
    effect: FunctionEffect,
    expression: &Expression,
) -> Result<(), ExpressionError> {
    let allowed = match operation {
        CollectionOperation::Map => matches!(effect, FunctionEffect::Pure | FunctionEffect::Emit),
        CollectionOperation::Filter | CollectionOperation::Fold => effect == FunctionEffect::Pure,
    };
    allowed.then_some(()).ok_or_else(|| {
        ExpressionError::new(
            "EXPRESSION_COLLECTION_CALLBACK_EFFECT",
            format!(
                "{} does not accept an `effect {effect}` callback",
                operation.as_str()
            ),
            expression.span.clone(),
        )
    })
}

pub(super) fn callback_parameter(
    callback: &TypedNode,
    index: usize,
    arity: usize,
    expression: &Expression,
) -> Result<ValueType, ExpressionError> {
    let ValueTypeKind::Function { parameters, .. } = callback.value_type.kind() else {
        return Err(type_error(
            format!("expected function value, found {}", callback.value_type),
            expression,
        ));
    };
    if parameters.len() != arity {
        return Err(type_error(
            "collection callback signature does not match its input",
            expression,
        ));
    }
    Ok(parameters[index].clone())
}

pub(in crate::program::expression::compile::lower) fn list_type(
    element: ValueType,
    expression: &Expression,
) -> Result<ValueType, ExpressionError> {
    ValueType::list(element).map_err(|error| type_error(error.message(), expression))
}

pub(super) fn type_error(message: impl Into<String>, expression: &Expression) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_CALL_ARGUMENT_TYPE",
        message,
        expression.span.clone(),
    )
}
