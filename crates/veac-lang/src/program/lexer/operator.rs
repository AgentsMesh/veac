use super::{Lexer, TokenKind};

impl Lexer<'_> {
    pub(super) fn arrow(&mut self) {
        self.pair(TokenKind::Arrow);
    }

    pub(super) fn one_or_two(&mut self, single: TokenKind, expected: char, paired: TokenKind) {
        if self.next() == Some(expected) {
            self.pair(paired);
        } else {
            self.single(single);
        }
    }

    pub(super) fn required_pair(&mut self, expected: char, paired: TokenKind) {
        if self.next() == Some(expected) {
            self.pair(paired);
        } else {
            let value = self.current().expect("operator starts at a character");
            self.invalid(value, self.offset);
        }
    }

    fn pair(&mut self, kind: TokenKind) {
        let start = self.offset;
        self.advance();
        self.advance();
        self.push(kind, start, self.offset);
    }
}
