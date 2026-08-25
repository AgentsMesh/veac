use crate::authoring::Span;
use crate::program::syntax_document::{SyntaxElement, SyntaxElementKind, TriviaKind};

use super::Lexer;

impl Lexer<'_> {
    pub(super) fn line_comment(&mut self) {
        let start = self.offset;
        self.take_while(|value| value != '\n');
        self.element(SyntaxElementKind::Trivia(TriviaKind::LineComment), start);
    }

    pub(super) fn block_comment(&mut self) {
        let start = self.offset;
        self.advance();
        self.advance();
        while self.current().is_some() {
            if self.current() == Some('*') && self.next() == Some('/') {
                self.advance();
                self.advance();
                self.element(SyntaxElementKind::Trivia(TriviaKind::BlockComment), start);
                return;
            }
            self.advance();
        }
        self.invalid('/', start);
    }

    pub(super) fn whitespace(&mut self) {
        let start = self.offset;
        self.take_while(char::is_whitespace);
        self.element(SyntaxElementKind::Gap, start);
    }

    fn element(&mut self, kind: SyntaxElementKind, start: usize) {
        self.elements.push(SyntaxElement {
            kind,
            span: Span {
                start,
                end: self.offset,
            },
        });
    }
}
