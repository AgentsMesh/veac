use super::Parser;
use crate::program::expression::ast::{Expression, ExpressionKind, MapEntry};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::ExpressionError;

impl Parser {
    pub(super) fn list(
        &mut self,
        opening: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.enter_depth(opening.clone())?;
        let result = self.list_contents(opening.start);
        self.depth -= 1;
        result
    }

    fn list_contents(&mut self, start: usize) -> Result<Expression, ExpressionError> {
        let values = self.comma_separated(&TokenKind::RightBracket)?;
        let closing = self.expect(&TokenKind::RightBracket, "]")?;
        self.node(ExpressionKind::List(values), start..closing.end)
    }

    pub(super) fn map(
        &mut self,
        opening: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.enter_depth(opening.clone())?;
        let result = self.map_contents(opening.start);
        self.depth -= 1;
        result
    }

    fn map_contents(&mut self, start: usize) -> Result<Expression, ExpressionError> {
        let mut entries = Vec::new();
        while !self.at(&TokenKind::RightBrace) {
            let key = self.expression()?;
            self.expect(&TokenKind::Colon, ":")?;
            let value = self.expression()?;
            entries.push(MapEntry { key, value });
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            self.reject_trailing(&TokenKind::RightBrace)?;
        }
        let closing = self.expect(&TokenKind::RightBrace, "}")?;
        self.node(ExpressionKind::Map(entries), start..closing.end)
    }

    pub(super) fn parenthesized_contents(
        &mut self,
        opening: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        if self.at(&TokenKind::RightParen) {
            let closing = self.advance();
            return Err(ExpressionError::new(
                "EXPRESSION_TUPLE_ARITY",
                "tuple literal requires at least two elements",
                opening.start..closing.span.end,
            ));
        }
        let first = self.expression()?;
        if self.take(&TokenKind::Comma).is_none() {
            self.expect(&TokenKind::RightParen, ")")?;
            return Ok(first);
        }
        if self.at(&TokenKind::RightParen) {
            return Err(self.error(
                "EXPRESSION_TUPLE_ARITY",
                "tuple literal requires at least two elements",
            ));
        }
        let mut values = vec![first];
        loop {
            values.push(self.expression()?);
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            self.reject_trailing(&TokenKind::RightParen)?;
        }
        let closing = self.expect(&TokenKind::RightParen, ")")?;
        self.node(ExpressionKind::Tuple(values), opening.start..closing.end)
    }

    fn comma_separated(&mut self, closing: &TokenKind) -> Result<Vec<Expression>, ExpressionError> {
        let mut values = Vec::new();
        while !self.at(closing) {
            values.push(self.expression()?);
            if self.take(&TokenKind::Comma).is_none() {
                break;
            }
            self.reject_trailing(closing)?;
        }
        Ok(values)
    }

    fn reject_trailing(&self, closing: &TokenKind) -> Result<(), ExpressionError> {
        if self.at(closing) {
            Err(self.error(
                "EXPRESSION_EXPECTED_VALUE",
                "trailing collection separators are not supported",
            ))
        } else {
            Ok(())
        }
    }
}
