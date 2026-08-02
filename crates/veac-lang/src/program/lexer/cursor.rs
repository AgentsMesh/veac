use super::Lexer;

impl Lexer<'_> {
    pub(super) fn take_while(&mut self, predicate: impl Fn(char) -> bool) {
        while self.current().is_some_and(&predicate) {
            self.advance();
        }
    }

    pub(super) fn current(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    pub(super) fn next(&self) -> Option<char> {
        self.source[self.offset..].chars().nth(1)
    }

    pub(super) fn advance(&mut self) {
        self.offset += self.current().map_or(0, char::len_utf8);
    }
}
