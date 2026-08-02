use super::Lexer;

impl Lexer<'_> {
    pub(super) fn block_comment(&mut self) -> bool {
        self.advance();
        self.advance();
        while self.current().is_some() {
            if self.current() == Some('*') && self.next() == Some('/') {
                self.advance();
                self.advance();
                return true;
            }
            self.advance();
        }
        false
    }

    pub(super) fn line_comment(&mut self) {
        while self.current().is_some_and(|character| character != '\n') {
            self.advance();
        }
    }

    pub(super) fn starts_signed_number(&self) -> bool {
        matches!(self.current(), Some('-' | '+'))
            && self.next().is_some_and(|value| value.is_ascii_digit())
    }

    pub(super) fn current(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    pub(super) fn next(&self) -> Option<char> {
        let mut chars = self.source[self.offset..].chars();
        chars.next()?;
        chars.next()
    }

    pub(super) fn advance(&mut self) {
        if let Some(character) = self.current() {
            self.offset += character.len_utf8();
        }
    }
}
