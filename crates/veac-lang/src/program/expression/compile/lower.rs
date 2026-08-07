use std::sync::Arc;

use super::super::ast::{Expression, ExpressionKind};
use super::super::hir::{TypedExpression, TypedNode, TypedNodeKind};
use super::super::{ExpressionContext, ExpressionError, ValueType};
use super::signature::FunctionSignatures;
use super::typing;

mod binary;
mod call;
mod closure;
mod collection;
mod collection_call;
mod control;
mod domain;
mod iteration;
mod r#match;
mod method;
mod model;
mod nominal;
mod path;
mod range;
mod scope;
mod temporal_attachment;
mod type_annotation;

use model::Lowerer;

#[cfg(test)]
#[path = "lower/range_tests.rs"]
mod range_tests;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(super) enum SymbolTarget {
    External(ValueType),
    TrustedExternal(ValueType),
    Parameter(usize, ValueType),
}

pub(super) fn expression(
    expression: &Expression,
    symbols: &dyn Fn(&str) -> Option<SymbolTarget>,
    context: &ExpressionContext,
) -> Result<TypedExpression, ExpressionError> {
    expression_with_signatures(expression, symbols, context, &FunctionSignatures::new())
}

fn expression_with_signatures(
    expression: &Expression,
    symbols: &dyn Fn(&str) -> Option<SymbolTarget>,
    context: &ExpressionContext,
    signatures: &FunctionSignatures,
) -> Result<TypedExpression, ExpressionError> {
    let types = context.types_arc();
    let domain = context.domain_arc();
    let mut lowerer = Lowerer {
        symbols,
        functions: context.functions(),
        signatures,
        methods: context.methods(),
        scopes: Vec::new(),
        closures: Vec::new(),
        initializers: Vec::new(),
        next_local: 0,
        next_mutable: 0,
        types: Arc::clone(&types),
        domain: &domain,
        static_values: context.static_values(),
        provisional_values: context.provisional_values(),
    };
    Ok(TypedExpression {
        root: lowerer.lower(expression)?,
        types,
        domain,
    })
}

pub(super) fn function_with_signatures(
    expression: &Expression,
    return_type: &ValueType,
    symbols: &dyn Fn(&str) -> Option<SymbolTarget>,
    context: &ExpressionContext,
    signatures: &FunctionSignatures,
) -> Result<TypedExpression, ExpressionError> {
    let types = context.types_arc();
    let domain = context.domain_arc();
    let mut lowerer = Lowerer {
        symbols,
        functions: context.functions(),
        signatures,
        methods: context.methods(),
        scopes: Vec::new(),
        closures: Vec::new(),
        initializers: Vec::new(),
        next_local: 0,
        next_mutable: 0,
        types: Arc::clone(&types),
        domain: &domain,
        static_values: context.static_values(),
        provisional_values: context.provisional_values(),
    };
    Ok(TypedExpression {
        root: lowerer.lower_context(expression, Some(return_type))?,
        types,
        domain,
    })
}

impl Lowerer<'_> {
    fn lower(&mut self, expression: &Expression) -> Result<TypedNode, ExpressionError> {
        self.lower_context(expression, None)
    }

    fn lower_context(
        &mut self,
        expression: &Expression,
        expected: Option<&ValueType>,
    ) -> Result<TypedNode, ExpressionError> {
        let (kind, value_type) = match &expression.kind {
            ExpressionKind::Literal(value) => {
                (TypedNodeKind::Literal(value.clone()), value.value_type())
            }
            ExpressionKind::Symbol(name) => self.symbol(name, expression)?,
            ExpressionKind::Unary { operator, operand } => {
                let operand = self.lower(operand)?;
                let value_type =
                    typing::unary(*operator, &operand.value_type, expression.span.clone())?;
                (
                    TypedNodeKind::Unary {
                        operator: *operator,
                        operand: Box::new(operand),
                    },
                    value_type,
                )
            }
            ExpressionKind::Binary {
                operator,
                left,
                right,
            } => self.binary(*operator, left, right, expression)?,
            ExpressionKind::Range { start, end, step } => {
                self.range(start, end, step.as_deref())?
            }
            ExpressionKind::Closure {
                parameters,
                return_type,
                effect,
                body,
            } => self.closure(parameters, return_type, *effect, body, expression)?,
            ExpressionKind::For {
                binding,
                binding_span,
                iterable,
                body,
            } => self.iteration(binding, binding_span, iterable, body, expected, expression)?,
            ExpressionKind::Call { callee, arguments } => {
                self.call(callee, arguments, expression)?
            }
            ExpressionKind::FieldProject {
                receiver,
                field,
                field_span,
            } => self.field_project(receiver, field, field_span, expression)?,
            ExpressionKind::NominalConstruct { path, fields } => {
                self.nominal_construct(path, fields, expression)?
            }
            ExpressionKind::Match { scrutinee, arms } => {
                self.match_expression(scrutinee, arms, expected, expression)?
            }
            ExpressionKind::TemporalAttach(value) => self.temporal_attachment(value, expression)?,
            ExpressionKind::List(values) => self.list(values, expected, expression)?,
            ExpressionKind::Map(entries) => self.map(entries, expected, expression)?,
            ExpressionKind::Tuple(values) => self.tuple(values, expected, expression)?,
            ExpressionKind::Block(block) => {
                let block = self.block(block, expected)?;
                let value_type = block.result.value_type.clone();
                (TypedNodeKind::Block(block), value_type)
            }
            ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.conditional(condition, then_branch, else_branch, expected, expression)?,
        };
        Ok(TypedNode {
            kind,
            value_type,
            span: expression.span.clone(),
        })
    }
}
