use super::{Lexer, TokenKind};

impl Lexer<'_> {
    pub(super) fn number(&mut self, start: usize) {
        while let Some(value) = self.current() {
            if value == '.' && self.next() == Some('.') {
                break;
            }
            if value.is_alphanumeric() || matches!(value, '.' | '%') {
                self.advance();
            } else {
                break;
            }
        }
        self.push(
            TokenKind::Number(self.source[start..self.offset].to_owned()),
            start,
            self.offset,
        );
    }
}
