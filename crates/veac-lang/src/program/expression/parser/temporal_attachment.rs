use super::Parser;
use crate::program::expression::ast::{Expression, ExpressionKind, TemporalAttachment};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::{ExpressionError, MAX_CALL_ARGUMENTS};

impl Parser {
    pub(super) fn temporal_attachment(
        &mut self,
        opening: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        let property = match self.advance() {
            crate::program::expression::lexer::Token {
                kind: TokenKind::Symbol(value),
                span,
            } => crate::program::model::TemporalProperty::parse(&value).ok_or_else(|| {
                ExpressionError::new(
                    "EXPRESSION_TEMPORAL_PROPERTY",
                    "expected a closed temporal property after `animate`",
                    span,
                )
            })?,
            token => {
                return Err(ExpressionError::new(
                    "EXPRESSION_TEMPORAL_PROPERTY",
                    "expected a closed temporal property after `animate`",
                    token.span,
                ))
            }
        };
        let on = self.advance();
        if !matches!(&on.kind, TokenKind::Symbol(value) if value == "on") {
            return Err(ExpressionError::new(
                "EXPRESSION_EXPECTED_TOKEN",
                "expected `on`",
                on.span,
            ));
        }
        let target = match self.advance() {
            crate::program::expression::lexer::Token {
                kind: TokenKind::Symbol(value),
                span,
            } => {
                crate::program::expression::TemporalTargetKind::parse(&value).ok_or_else(|| {
                    ExpressionError::new(
                        "EXPRESSION_TEMPORAL_TARGET",
                        "expected a closed temporal target kind after `on`",
                        span,
                    )
                })?
            }
            token => {
                return Err(ExpressionError::new(
                    "EXPRESSION_TEMPORAL_TARGET",
                    "expected a closed temporal target kind after `on`",
                    token.span,
                ))
            }
        };
        let arguments = self.temporal_target_arguments()?;
        let body_open = self.expect(&TokenKind::LeftBrace, "{")?;
        let body_start = body_open.start;
        let (body, span) = self.block(body_open)?;
        self.node(
            ExpressionKind::TemporalAttach(Box::new(TemporalAttachment {
                property,
                target,
                arguments,
                body,
                body_span: body_start..span.end,
            })),
            opening.start..span.end,
        )
    }

    fn temporal_target_arguments(&mut self) -> Result<Vec<Expression>, ExpressionError> {
        let opening = self.expect(&TokenKind::LeftParen, "(")?;
        self.enter_depth(opening)?;
        let mut arguments = Vec::new();
        while !self.at(&TokenKind::RightParen) {
            let argument = self.expression()?;
            if arguments.len() >= MAX_CALL_ARGUMENTS {
                return Err(ExpressionError::new(
                    "EXPRESSION_CALL_ARGUMENT_LIMIT",
                    format!("temporal target exceeds the {MAX_CALL_ARGUMENTS} argument limit"),
                    argument.span,
                ));
            }
            arguments.push(argument);
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            if self.at(&TokenKind::RightParen) {
                return Err(self.error(
                    "EXPRESSION_EXPECTED_VALUE",
                    "trailing temporal target separators are not supported",
                ));
            }
        }
        self.expect(&TokenKind::RightParen, ")")?;
        self.depth -= 1;
        Ok(arguments)
    }
}
