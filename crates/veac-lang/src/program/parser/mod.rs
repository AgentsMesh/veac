mod component;
mod declaration;
mod file;
mod instance;
pub(crate) mod kind;
mod name;

use crate::authoring::Span;

use super::diagnostic::Diagnostic;
use super::model::RawBlock;
use super::token::{Token, TokenKind};

pub(crate) use file::parse;

pub(super) struct Parser<'a> {
    path: &'a str,
    source: &'a str,
    tokens: Vec<Token>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn new(path: &'a str, source: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            path,
            source,
            tokens,
            cursor: 0,
        }
    }

    pub(super) fn at_word(&self, expected: &str) -> bool {
        self.current().word() == Some(expected)
    }

    pub(super) fn at(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    pub(super) fn at_eof(&self) -> bool {
        self.at(&TokenKind::Eof)
    }

    pub(super) fn current(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    pub(super) fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !self.at_eof() {
            self.cursor += 1;
        }
        token
    }

    pub(super) fn word(&mut self, context: &str) -> Result<(String, Span), Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::Word(value) => Ok((value, token.span)),
            _ => Err(self.error(
                "PROGRAM_EXPECTED_WORD",
                format!("expected {context}"),
                token.span,
            )),
        }
    }

    pub(super) fn local_id(&mut self, context: &str) -> Result<(String, Span), Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::LocalId(value) if crate::name::is_name(&value) => Ok((value, token.span)),
            TokenKind::LocalId(_) => Err(self.error(
                "PROGRAM_IDENTIFIER",
                format!("{context} must be {}", crate::name::NAME_CONTRACT),
                token.span,
            )),
            _ => Err(self.error(
                "PROGRAM_EXPECTED_LOCAL_ID",
                format!("expected @{context}"),
                token.span,
            )),
        }
    }

    pub(super) fn string(&mut self, context: &str) -> Result<(String, Span), Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::String(value) => Ok((value, token.span)),
            _ => Err(self.error(
                "PROGRAM_EXPECTED_STRING",
                format!("expected {context}"),
                token.span,
            )),
        }
    }

    pub(super) fn expect_word(&mut self, expected: &str) -> Result<Span, Diagnostic> {
        if self.at_word(expected) {
            Ok(self.advance().span)
        } else {
            Err(self.error(
                "PROGRAM_EXPECTED_WORD",
                format!("expected `{expected}`"),
                self.current().span,
            ))
        }
    }

    pub(super) fn expect(&mut self, kind: TokenKind, label: &str) -> Result<Span, Diagnostic> {
        if self.at(&kind) {
            Ok(self.advance().span)
        } else {
            Err(self.error(
                "PROGRAM_EXPECTED_TOKEN",
                format!("expected {label}"),
                self.current().span,
            ))
        }
    }

    pub(super) fn expression_until_semicolon(&mut self) -> Result<(String, Span), Diagnostic> {
        let start = self.current().span.start;
        let mut parentheses = 0usize;
        while !self.at_eof() {
            match self.current().kind {
                TokenKind::LeftParen => parentheses += 1,
                TokenKind::RightParen if parentheses > 0 => parentheses -= 1,
                TokenKind::Semicolon if parentheses == 0 => break,
                _ => {}
            }
            self.advance();
        }
        let end = self.current().span.start;
        self.expect(TokenKind::Semicolon, "`;`")?;
        let raw = &self.source[start..end];
        let leading = raw.len() - raw.trim_start().len();
        let value = raw.trim().to_owned();
        let span = Span {
            start: start + leading,
            end: start + leading + value.len(),
        };
        if value.is_empty() {
            return Err(self.error(
                "PROGRAM_EMPTY_EXPRESSION",
                "expression cannot be empty",
                Span { start, end },
            ));
        }
        Ok((value, span))
    }

    pub(super) fn raw_block(&mut self) -> Result<RawBlock, Diagnostic> {
        let left = self.expect(TokenKind::LeftBrace, "`{`")?;
        let mut depth = 1usize;
        while depth > 0 && !self.at_eof() {
            let token = self.advance();
            match token.kind {
                TokenKind::LeftBrace | TokenKind::DollarLeftBrace => depth += 1,
                TokenKind::RightBrace => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                return Ok(RawBlock {
                    content_span: Span {
                        start: left.end,
                        end: token.span.start,
                    },
                    span: left.join(token.span),
                });
            }
        }
        Err(self.error("PROGRAM_UNCLOSED_BLOCK", "block is not closed", left))
    }

    pub(super) fn error(
        &self,
        code: &'static str,
        message: impl Into<String>,
        span: Span,
    ) -> Diagnostic {
        Diagnostic::new(code, self.path, message, span)
    }
}
