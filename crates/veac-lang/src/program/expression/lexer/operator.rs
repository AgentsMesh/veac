use super::{Lexer, Token, TokenKind};
use crate::program::expression::ExpressionError;

impl Lexer<'_> {
    pub(super) fn one_or_two(&mut self, single: TokenKind, expected: char, paired: TokenKind) {
        if self.next() == Some(expected) {
            self.pair(paired);
        } else {
            self.single(single);
        }
    }

    pub(super) fn required_pair(
        &mut self,
        expected: char,
        paired: TokenKind,
    ) -> Result<(), ExpressionError> {
        if self.next() == Some(expected) {
            self.pair(paired);
            Ok(())
        } else {
            let start = self.offset;
            let value = self.current().expect("operator starts at a character");
            self.advance();
            Err(ExpressionError::new(
                "EXPRESSION_LEX_OPERATOR",
                format!("operator `{value}` must be followed by `{expected}`"),
                start..self.offset,
            ))
        }
    }

    pub(super) fn pair(&mut self, kind: TokenKind) {
        let start = self.offset;
        self.advance();
        self.advance();
        self.tokens.push(Token {
            kind,
            span: start..self.offset,
        });
    }
}
