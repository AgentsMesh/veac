use std::collections::BTreeSet;
use std::ops::Range;

use super::Parser;
use crate::program::expression::ast::{Expression, ExpressionKind, NominalField};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::ExpressionError;
use crate::program::MAX_TYPE_MEMBERS;

impl Parser {
    pub(super) fn nominal_construct(
        &mut self,
        target: Expression,
        opening: Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        let path = crate::program::expression::ast::static_path(&target)
            .expect("nominal target is a static path");
        self.enter_depth(opening)?;
        let result = self.nominal_fields(target.span.start, path);
        self.depth -= 1;
        result
    }

    fn nominal_fields(
        &mut self,
        start: usize,
        path: Vec<crate::program::expression::ast::PathSegment>,
    ) -> Result<Expression, ExpressionError> {
        let mut fields = Vec::new();
        let mut names = BTreeSet::new();
        while !self.at(&TokenKind::RightBrace) {
            if fields.len() >= MAX_TYPE_MEMBERS {
                return Err(self.error(
                    "EXPRESSION_NOMINAL_FIELD_LIMIT",
                    format!("nominal construction exceeds {MAX_TYPE_MEMBERS} fields"),
                ));
            }
            let token = self.advance();
            let TokenKind::Symbol(name) = token.kind else {
                return Err(ExpressionError::new(
                    "EXPRESSION_NOMINAL_FIELD_NAME",
                    "expected a nominal field name",
                    token.span,
                ));
            };
            if !names.insert(name.clone()) {
                return Err(ExpressionError::new(
                    "EXPRESSION_DUPLICATE_NOMINAL_FIELD",
                    format!("nominal field `{name}` is authored more than once"),
                    token.span,
                ));
            }
            self.expect(&TokenKind::Colon, ":")?;
            let value = self.expression()?;
            fields.push(NominalField {
                name,
                name_span: token.span,
                value,
            });
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            if self.at(&TokenKind::RightBrace) {
                break;
            }
        }
        let closing = self.expect(&TokenKind::RightBrace, "}")?;
        self.node(
            ExpressionKind::NominalConstruct { path, fields },
            start..closing.end,
        )
    }
}

#[cfg(test)]
#[path = "nominal/tests.rs"]
mod tests;
