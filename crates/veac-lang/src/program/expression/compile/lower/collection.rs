use super::Lowerer;
use crate::program::expression::ast::Expression;
use crate::program::expression::hir::{TypedNode, TypedNodeKind};
use crate::program::expression::{ExpressionError, ValueType, ValueTypeError, ValueTypeKind};

mod map;

impl Lowerer<'_> {
    pub(super) fn list(
        &mut self,
        values: &[Expression],
        expected: Option<&ValueType>,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let expected = expected.and_then(list_element);
        if values.is_empty() {
            let element = expected.ok_or_else(|| context_error("list", expression))?;
            return Ok((
                TypedNodeKind::List(Vec::new()),
                construct_list(element, expression)?,
            ));
        }
        let (element, mut lowered, start) = match expected {
            Some(element) => (element.clone(), Vec::with_capacity(values.len()), 0),
            None => {
                let first = self.lower(&values[0])?;
                let element = first.value_type.clone();
                (element, vec![first], 1)
            }
        };
        for value in &values[start..] {
            let value = self.lower_context(value, Some(&element))?;
            require_type(
                &value,
                &element,
                "EXPRESSION_LIST_ELEMENT_TYPE",
                "list element",
            )?;
            lowered.push(value);
        }
        let value_type = construct_list(&element, expression)?;
        Ok((TypedNodeKind::List(lowered), value_type))
    }

    pub(super) fn tuple(
        &mut self,
        values: &[Expression],
        expected: Option<&ValueType>,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let expected = expected.and_then(|value| match value.kind() {
            ValueTypeKind::Tuple(elements) if elements.len() == values.len() => Some(elements),
            _ => None,
        });
        let mut lowered = Vec::with_capacity(values.len());
        for (index, value) in values.iter().enumerate() {
            let wanted = expected.and_then(|elements| elements.get(index));
            let value = self.lower_context(value, wanted)?;
            if let Some(wanted) = wanted {
                require_type(
                    &value,
                    wanted,
                    "EXPRESSION_TUPLE_ELEMENT_TYPE",
                    "tuple element",
                )?;
            }
            lowered.push(value);
        }
        let types = lowered
            .iter()
            .map(|value| value.value_type.clone())
            .collect();
        let value_type = ValueType::tuple(types).map_err(|error| type_error(error, expression))?;
        Ok((TypedNodeKind::Tuple(lowered), value_type))
    }
}

fn list_element(value: &ValueType) -> Option<&ValueType> {
    match value.kind() {
        ValueTypeKind::List(element) => Some(element),
        _ => None,
    }
}

pub(super) fn require_type(
    value: &TypedNode,
    expected: &ValueType,
    code: &'static str,
    role: &str,
) -> Result<(), ExpressionError> {
    if &value.value_type == expected {
        Ok(())
    } else {
        Err(ExpressionError::new(
            code,
            format!("{role} expects {expected}, found {}", value.value_type),
            value.span.clone(),
        ))
    }
}

pub(super) fn context_error(kind: &str, expression: &Expression) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_COLLECTION_TYPE_CONTEXT",
        format!("empty {kind} literal requires an expected type"),
        expression.span.clone(),
    )
}

fn construct_list(
    element: &ValueType,
    expression: &Expression,
) -> Result<ValueType, ExpressionError> {
    ValueType::list(element.clone()).map_err(|error| type_error(error, expression))
}

fn type_error(error: ValueTypeError, expression: &Expression) -> ExpressionError {
    ExpressionError::new("EXPRESSION_TYPE", error.message(), expression.span.clone())
}
