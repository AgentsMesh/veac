use super::Parser;
use crate::program::expression::ast::{
    CallArgument, CallArgumentLabel, Expression, ExpressionKind,
};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::{ExpressionError, MAX_CALL_ARGUMENTS};

impl Parser {
    pub(super) fn postfix(&mut self) -> Result<Expression, ExpressionError> {
        let mut callee = self.primary()?;
        loop {
            if let Some(dot) = self.take(&TokenKind::Dot) {
                callee = self.postfix_field(callee, dot)?;
            } else if let Some(opening) = self.take(&TokenKind::LeftParen) {
                callee = self.postfix_call(callee, opening)?;
            } else if self.nominal_allowed()
                && self.at(&TokenKind::LeftBrace)
                && crate::program::expression::ast::static_path(&callee).is_some()
            {
                let opening = self.advance().span;
                callee = self.nominal_construct(callee, opening)?;
            } else {
                break;
            }
        }
        Ok(callee)
    }

    fn postfix_field(
        &mut self,
        receiver: Expression,
        dot: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        let token = self.advance();
        let TokenKind::Symbol(field) = token.kind else {
            return Err(ExpressionError::new(
                "EXPRESSION_FIELD_NAME",
                "expected a field or path segment after `.`",
                dot.start..token.span.end,
            ));
        };
        let span = receiver.span.start..token.span.end;
        self.node(
            ExpressionKind::FieldProject {
                receiver: Box::new(receiver),
                field,
                field_span: token.span,
            },
            span,
        )
    }

    fn postfix_call(
        &mut self,
        callee: Expression,
        opening: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.enter_depth(opening)?;
        let mut arguments = Vec::new();
        while !self.at(&TokenKind::RightParen) {
            let label = if matches!(self.current().kind, TokenKind::Symbol(_))
                && self.next_at(&TokenKind::Colon)
            {
                let token = self.advance();
                let TokenKind::Symbol(name) = token.kind else {
                    unreachable!("named argument lookahead requires a symbol")
                };
                self.advance();
                Some(CallArgumentLabel {
                    name,
                    span: token.span,
                })
            } else {
                None
            };
            let value = self.expression()?;
            if arguments.len() >= MAX_CALL_ARGUMENTS {
                return Err(ExpressionError::new(
                    "EXPRESSION_CALL_ARGUMENT_LIMIT",
                    format!("function call exceeds the {MAX_CALL_ARGUMENTS} argument limit"),
                    value.span,
                ));
            }
            arguments.push(CallArgument { label, value });
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            if self.at(&TokenKind::RightParen) {
                break;
            }
        }
        let closing = self.expect(&TokenKind::RightParen, ")")?;
        self.depth -= 1;
        let span = callee.span.start..closing.end;
        self.node(
            ExpressionKind::Call {
                callee: Box::new(callee),
                arguments,
            },
            span,
        )
    }
}

#[cfg(test)]
#[path = "postfix/tests.rs"]
mod tests;
