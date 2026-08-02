use super::Lexer;
use crate::program::expression::ExpressionError;

impl Lexer<'_> {
    pub(super) fn line_comment(&mut self) {
        while self.current().is_some_and(|value| value != '\n') {
            self.advance();
        }
    }

    pub(super) fn block_comment(&mut self) -> Result<(), ExpressionError> {
        let start = self.offset;
        self.advance();
        self.advance();
        while self.current().is_some() {
            if self.current() == Some('*') && self.next() == Some('/') {
                self.advance();
                self.advance();
                return Ok(());
            }
            self.advance();
        }
        Err(ExpressionError::new(
            "EXPRESSION_BLOCK_COMMENT",
            "block comment is not closed",
            start..self.offset,
        ))
    }
}
