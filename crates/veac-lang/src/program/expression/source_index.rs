mod compound;
mod control;

use std::ops::Range;

use crate::source_edit::{SourceExpressionPath, SourceExpressionStep};

use super::ast::{Block, Expression, ExpressionKind, Statement};
use super::{lexer, parser, ExpressionError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexedSourceExpression {
    pub path: SourceExpressionPath,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexedSourceStatement {
    pub path: SourceExpressionPath,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct IndexedBodySites {
    pub expressions: Vec<IndexedSourceExpression>,
    pub statements: Vec<IndexedSourceStatement>,
}

pub(crate) fn indexed_body_sites(source: &str) -> Result<IndexedBodySites, ExpressionError> {
    let root = parser::parse(lexer::lex(source)?)?;
    let ExpressionKind::Block(block) = &root.kind else {
        return Err(ExpressionError::new(
            "EXPRESSION_FUNCTION_BODY",
            "function body must be a block expression",
            root.span,
        ));
    };
    let mut walker = Walker::default();
    walker.block(block, &mut Vec::new());
    if let Some(span) = invalid_path_span(&walker.output) {
        return Err(ExpressionError::new(
            "EXPRESSION_SOURCE_PATH",
            "expression produced an invalid semantic source path",
            span,
        ));
    }
    Ok(walker.output)
}

fn invalid_path_span(sites: &IndexedBodySites) -> Option<Range<usize>> {
    sites
        .expressions
        .iter()
        .find(|entry| !entry.path.is_valid())
        .map(|entry| entry.span.clone())
        .or_else(|| {
            sites
                .statements
                .iter()
                .find(|entry| !entry.path.is_valid() || !entry.path.ends_in_local_value())
                .map(|entry| entry.span.clone())
        })
}

#[derive(Default)]
struct Walker {
    output: IndexedBodySites,
}

impl Walker {
    fn block(&mut self, block: &Block, path: &mut Vec<SourceExpressionStep>) {
        for (ordinal, statement) in block.statements.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).expect("expression node limit fits u32");
            let (operation, binding, value, span) = match statement {
                Statement::Let(value) => (
                    crate::source_edit::SourceLocalOperation::Let,
                    value.name.as_str(),
                    &value.value,
                    &value.span,
                ),
                Statement::Var(value) => (
                    crate::source_edit::SourceLocalOperation::Var,
                    value.name.as_str(),
                    &value.value,
                    &value.span,
                ),
                Statement::Set(value) => (
                    crate::source_edit::SourceLocalOperation::Set,
                    value.name.as_str(),
                    &value.value,
                    &value.span,
                ),
            };
            path.push(SourceExpressionStep::LocalValue {
                operation,
                binding: binding.to_owned(),
                ordinal,
            });
            self.output.statements.push(IndexedSourceStatement {
                path: SourceExpressionPath::new(path.clone()),
                span: span.clone(),
            });
            self.expression(value, path);
            path.pop();
        }
        self.child(&block.result, path, SourceExpressionStep::BlockResult);
    }

    fn child(
        &mut self,
        expression: &Expression,
        path: &mut Vec<SourceExpressionStep>,
        step: SourceExpressionStep,
    ) {
        path.push(step);
        self.expression(expression, path);
        path.pop();
    }

    fn expression(&mut self, expression: &Expression, path: &mut Vec<SourceExpressionStep>) {
        self.output.expressions.push(IndexedSourceExpression {
            path: SourceExpressionPath::new(path.clone()),
            span: expression.span.clone(),
        });
        match &expression.kind {
            ExpressionKind::Literal(_) | ExpressionKind::Symbol(_) => {}
            ExpressionKind::Unary { operand, .. } => {
                self.child(operand, path, SourceExpressionStep::UnaryOperand)
            }
            ExpressionKind::Binary { left, right, .. } => {
                self.child(left, path, SourceExpressionStep::BinaryLeft);
                self.child(right, path, SourceExpressionStep::BinaryRight);
            }
            ExpressionKind::Range { start, end, step } => {
                self.child(start, path, SourceExpressionStep::RangeStart);
                self.child(end, path, SourceExpressionStep::RangeEnd);
                if let Some(step) = step {
                    self.child(step, path, SourceExpressionStep::RangeStep);
                }
            }
            ExpressionKind::Closure { body, .. } => {
                path.push(SourceExpressionStep::ClosureBody);
                self.block(body, path);
                path.pop();
            }
            ExpressionKind::For { iterable, body, .. } => {
                self.child(iterable, path, SourceExpressionStep::IterationSource);
                path.push(SourceExpressionStep::IterationBody);
                self.block(body, path);
                path.pop();
            }
            ExpressionKind::Block(block) => self.block(block, path),
            other => self.compound(other, path),
        }
    }
}

#[cfg(test)]
#[path = "source_index/tests.rs"]
mod tests;
