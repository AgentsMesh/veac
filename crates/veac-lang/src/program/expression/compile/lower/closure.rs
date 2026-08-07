use std::collections::BTreeMap;

use super::scope::{ClosureContext, ScopeBinding};
use super::Lowerer;
use crate::program::expression::ast::{Block, ClosureParameter, Expression, TypeAnnotation};
use crate::program::expression::hir::{TypedClosureParameter, TypedNodeKind};
use crate::program::expression::{ExpressionError, FunctionEffect, Stage, ValueType};

impl Lowerer<'_> {
    pub(super) fn closure(
        &mut self,
        parameters: &[ClosureParameter],
        return_type: &TypeAnnotation,
        effect: FunctionEffect,
        body: &Block,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let parameter_types = parameters
            .iter()
            .map(|value| self.resolve_annotation(&value.annotation))
            .collect::<Result<Vec<_>, _>>()?;
        let return_type = self.resolve_annotation(return_type)?;
        let value_type = ValueType::function(parameter_types.clone(), return_type.clone(), effect)
            .map_err(|error| {
                ExpressionError::new(
                    "EXPRESSION_TYPE_SYNTAX",
                    error.message(),
                    expression.span.clone(),
                )
            })?;
        let checkpoint = self.checkpoint();
        self.closures.push(ClosureContext::default());
        let owner = self.closures.len();
        self.scopes
            .push(parameter_scope(parameters, &parameter_types, owner));
        let lowered = self.block(body, Some(&return_type));
        self.scopes.pop();
        let captures = self
            .closures
            .pop()
            .expect("closure context is active")
            .captures;
        let body = match lowered {
            Ok(body) => body,
            Err(error) => {
                self.rollback(&checkpoint);
                return Err(error);
            }
        };
        if body.result.value_type != return_type {
            return Err(ExpressionError::new(
                "EXPRESSION_CLOSURE_RETURN_TYPE",
                format!(
                    "closure declares {}, but its body has type {}",
                    return_type, body.result.value_type
                ),
                body.result.span.clone(),
            ));
        }
        let parameters = parameter_types
            .into_iter()
            .map(|value_type| TypedClosureParameter {
                value_type,
                stage: Stage::Const,
            })
            .collect();
        Ok((
            TypedNodeKind::Closure {
                parameters,
                captures,
                body,
                non_escaping: false,
            },
            value_type,
        ))
    }
}

fn parameter_scope(
    parameters: &[ClosureParameter],
    types: &[ValueType],
    owner: usize,
) -> BTreeMap<String, ScopeBinding> {
    parameters
        .iter()
        .zip(types)
        .enumerate()
        .map(|(index, (value, value_type))| {
            (
                value.name.clone(),
                ScopeBinding::parameter(index, value_type.clone(), owner),
            )
        })
        .collect()
}

#[cfg(test)]
#[path = "closure/tests.rs"]
mod tests;
