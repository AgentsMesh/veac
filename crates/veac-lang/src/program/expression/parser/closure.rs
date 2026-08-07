use std::collections::BTreeSet;

use super::Parser;
use crate::program::expression::ast::{ClosureParameter, Expression, ExpressionKind};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::{ExpressionError, MAX_FUNCTION_PARAMETERS};
use crate::vocabulary::{accepts_expression_name, ExpressionNameKind};

impl Parser {
    pub(super) fn closure(
        &mut self,
        start: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.expect(&TokenKind::LeftParen, "(")?;
        let parameters = self.closure_parameters()?;
        self.expect(&TokenKind::RightParen, ")")?;
        self.expect(&TokenKind::Arrow, "->")?;
        let return_type = self.type_annotation()?;
        let (effect, _) = self.function_effect()?;
        let opening = self.expect(&TokenKind::LeftBrace, "{")?;
        let (body, span) = self.block(opening)?;
        self.node(
            ExpressionKind::Closure {
                parameters,
                return_type: Box::new(return_type),
                effect,
                body,
            },
            start.start..span.end,
        )
    }

    fn closure_parameters(&mut self) -> Result<Vec<ClosureParameter>, ExpressionError> {
        let mut parameters = Vec::new();
        let mut names = BTreeSet::new();
        while !self.at(&TokenKind::RightParen) {
            let token = self.advance();
            let TokenKind::Symbol(name) = token.kind else {
                return Err(ExpressionError::new(
                    "EXPRESSION_PARAMETER_NAME",
                    "expected a closure parameter name",
                    token.span,
                ));
            };
            if !accepts_expression_name(&name, ExpressionNameKind::Parameter) {
                return Err(ExpressionError::new(
                    "EXPRESSION_PARAMETER_NAME",
                    format!("`{name}` is not a valid closure parameter name"),
                    token.span,
                ));
            }
            if !names.insert(name.clone()) {
                return Err(ExpressionError::new(
                    "EXPRESSION_DUPLICATE_PARAMETER",
                    format!("closure parameter `{name}` is declared more than once"),
                    token.span,
                ));
            }
            self.expect(&TokenKind::Colon, ":")?;
            let annotation = self.type_annotation()?;
            parameters.push(ClosureParameter {
                name,
                name_span: token.span,
                annotation,
            });
            if parameters.len() > MAX_FUNCTION_PARAMETERS {
                return Err(self.error(
                    "EXPRESSION_FUNCTION_PARAMETER_LIMIT",
                    format!("closure exceeds the {MAX_FUNCTION_PARAMETERS} parameter limit"),
                ));
            }
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            if self.at(&TokenKind::RightParen) {
                return Err(self.error(
                    "EXPRESSION_EXPECTED_VALUE",
                    "trailing closure parameter separators are not supported",
                ));
            }
        }
        Ok(parameters)
    }
}

#[cfg(test)]
#[path = "closure/tests.rs"]
mod tests;
