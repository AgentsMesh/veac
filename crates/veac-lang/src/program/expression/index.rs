use std::ops::Range;

use super::ast::{Block, Expression, ExpressionKind, Statement};
use super::{ExpressionError, TemporalTargetKind};
use crate::program::model::TemporalProperty;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexedTemporalAttachment {
    pub(crate) property: TemporalProperty,
    pub(crate) target: TemporalTargetKind,
    pub(crate) declaration: Range<usize>,
    pub(crate) body: Range<usize>,
}

pub(crate) fn indexed_temporal_attachments(
    source: &str,
) -> Result<Vec<IndexedTemporalAttachment>, ExpressionError> {
    let expression = super::parser::parse(super::lexer::lex(source)?)?;
    let mut output = Vec::new();
    collect(&expression, &mut output);
    output.sort_by_key(|value| value.declaration.start);
    Ok(output)
}

fn collect(expression: &Expression, output: &mut Vec<IndexedTemporalAttachment>) {
    match &expression.kind {
        ExpressionKind::Literal(_) | ExpressionKind::Symbol(_) => {}
        ExpressionKind::Unary { operand, .. } => collect(operand, output),
        ExpressionKind::Binary { left, right, .. } => {
            collect(left, output);
            collect(right, output);
        }
        ExpressionKind::Range { start, end, step } => {
            collect(start, output);
            collect(end, output);
            if let Some(step) = step {
                collect(step, output);
            }
        }
        ExpressionKind::Closure { body, .. } | ExpressionKind::For { body, .. } => {
            collect_block(body, output)
        }
        ExpressionKind::Call { callee, arguments } => {
            collect(callee, output);
            collect_many(arguments, output);
        }
        ExpressionKind::FieldProject { receiver, .. } => collect(receiver, output),
        ExpressionKind::NominalConstruct { fields, .. } => {
            collect_many(fields.iter().map(|field| &field.value), output)
        }
        ExpressionKind::Match { scrutinee, arms } => {
            collect(scrutinee, output);
            arms.iter().for_each(|arm| collect(&arm.body, output));
        }
        ExpressionKind::TemporalAttach(value) => {
            output.push(IndexedTemporalAttachment {
                property: value.property,
                target: value.target,
                declaration: expression.span.clone(),
                body: value.body_span.clone(),
            });
            collect_many(&value.arguments, output);
            collect_block(&value.body, output);
        }
        ExpressionKind::List(values) | ExpressionKind::Tuple(values) => {
            collect_many(values, output)
        }
        ExpressionKind::Map(entries) => {
            for entry in entries {
                collect_many([&entry.key, &entry.value], output);
            }
        }
        ExpressionKind::Block(value) => collect_block(value, output),
        ExpressionKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect(condition, output);
            collect_block(then_branch, output);
            collect_block(else_branch, output);
        }
    }
}

fn collect_block(value: &Block, output: &mut Vec<IndexedTemporalAttachment>) {
    for statement in &value.statements {
        match statement {
            Statement::Let(value) => collect(&value.value, output),
            Statement::Var(value) => collect(&value.value, output),
            Statement::Set(value) => collect(&value.value, output),
        }
    }
    collect(&value.result, output);
}

fn collect_many<'a>(
    values: impl IntoIterator<Item = &'a Expression>,
    output: &mut Vec<IndexedTemporalAttachment>,
) {
    values.into_iter().for_each(|value| collect(value, output));
}
