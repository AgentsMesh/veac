use std::collections::BTreeMap;

use super::collection_call::{iterable_element, list_type};
use super::scope::{ClosureContext, ScopeBinding};
use super::Lowerer;
use crate::program::expression::ast::{Block, Expression};
use crate::program::expression::hir::{
    TypedClosureParameter, TypedIteration, TypedNode, TypedNodeKind,
};
use crate::program::expression::{
    ExpressionError, FunctionEffect, Stage, ValueType, ValueTypeKind,
};

impl Lowerer<'_> {
    pub(super) fn iteration(
        &mut self,
        binding: &str,
        binding_span: &std::ops::Range<usize>,
        iterable: &Expression,
        body: &Block,
        expected: Option<&ValueType>,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let iterable_node = self.lower(iterable)?;
        let element = iterable_element(&iterable_node.value_type, expression)?;
        let body_expected = expected.and_then(list_element);
        let callback =
            self.iteration_callback(binding, element, body, body_expected, expression)?;
        let result = callback_result(&callback);
        let value_type = list_type(result, expression)?;
        Ok((
            TypedNodeKind::ForEach {
                iterable: Box::new(iterable_node),
                body: Box::new(callback),
                iteration: TypedIteration {
                    binding_span: binding_span.clone(),
                },
            },
            value_type,
        ))
    }

    fn iteration_callback(
        &mut self,
        binding: &str,
        element: ValueType,
        body: &Block,
        expected: Option<&ValueType>,
        expression: &Expression,
    ) -> Result<TypedNode, ExpressionError> {
        let checkpoint = self.checkpoint();
        self.closures.push(ClosureContext::iteration());
        let owner = self.closures.len();
        self.scopes.push(parameter_scope(binding, &element, owner));
        let lowered = self.block(body, expected);
        self.scopes.pop();
        let context = self.closures.pop().expect("iteration closure is active");
        let body = match lowered {
            Ok(body) => body,
            Err(error) => {
                self.rollback(&checkpoint);
                return Err(error);
            }
        };
        let result = body.result.value_type.clone();
        let value_type = ValueType::function(vec![element.clone()], result, FunctionEffect::Emit)
            .map_err(|error| {
            ExpressionError::new("EXPRESSION_TYPE", error.message(), expression.span.clone())
        })?;
        Ok(TypedNode {
            kind: TypedNodeKind::Closure {
                parameters: vec![TypedClosureParameter {
                    value_type: element,
                    stage: Stage::Const,
                }],
                captures: context.captures,
                body,
                non_escaping: true,
            },
            value_type,
            span: expression.span.clone(),
        })
    }
}

fn parameter_scope(
    binding: &str,
    element: &ValueType,
    owner: usize,
) -> BTreeMap<String, ScopeBinding> {
    BTreeMap::from([(
        binding.to_owned(),
        ScopeBinding::parameter(0, element.clone(), owner),
    )])
}

fn list_element(value_type: &ValueType) -> Option<&ValueType> {
    match value_type.kind() {
        ValueTypeKind::List(element) => Some(element),
        _ => None,
    }
}

fn callback_result(callback: &TypedNode) -> ValueType {
    let ValueTypeKind::Function { result, .. } = callback.value_type.kind() else {
        unreachable!("iteration callback is constructed with a function type")
    };
    result.clone()
}

#[cfg(test)]
#[path = "iteration/tests.rs"]
mod tests;
