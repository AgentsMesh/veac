use crate::authoring::Span;

use super::{Diagnostic, Lexer, Token, TokenKind};

const MAX_DIAGNOSTICS: usize = 256;

impl Lexer<'_> {
    pub(super) fn invalid(&mut self, value: char, start: usize) {
        if self.offset == start {
            self.advance();
        }
        self.report(Diagnostic::new(
            "PROGRAM_LEX_TOKEN",
            self.path,
            format!("unexpected or unterminated token starting with `{value}`"),
            Span {
                start,
                end: self.offset,
            },
        ));
    }

    pub(super) fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        if self.tokens.len() >= self.token_limit {
            if !self.token_limit_reported {
                self.report(Diagnostic::new(
                    "PROGRAM_TOKEN_LIMIT",
                    self.path,
                    "source module exceeds the one million token budget",
                    Span { start, end },
                ));
                self.token_limit_reported = true;
            }
            return;
        }
        self.tokens.push(Token {
            kind,
            span: Span { start, end },
        });
    }

    pub(super) fn report(&mut self, diagnostic: Diagnostic) {
        if self.diagnostics.len() < MAX_DIAGNOSTICS - 1 {
            self.diagnostics.push(diagnostic);
        } else if !self.diagnostic_limit_reported {
            self.diagnostics.push(Diagnostic::new(
                "PROGRAM_DIAGNOSTIC_LIMIT",
                self.path,
                "source module exceeds the diagnostic budget",
                diagnostic.span,
            ));
            self.diagnostic_limit_reported = true;
        }
    }
}
