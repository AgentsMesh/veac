use super::{Diagnostic, Lexer, Span, Token, TokenKind};

impl Lexer<'_> {
    pub(super) fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        if matches!(kind, TokenKind::Eof) {
            self.tokens.push(Token {
                kind,
                span: Span { start, end },
            });
            return;
        }
        if self.tokens.len() >= self.token_limit {
            if !self.token_limit_reported {
                self.report(Diagnostic {
                    code: "AUTHORING_TOKEN_LIMIT",
                    message: "authoring source exceeds one million tokens".to_owned(),
                    span: Span { start, end },
                });
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
        self.diagnostic_limit_reported |=
            crate::authoring::diagnostic_budget::push(&mut self.diagnostics, diagnostic);
    }
}
