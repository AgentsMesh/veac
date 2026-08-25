use std::ops::Range;

use super::super::lexer::{decode_string, validate_color, Token, TokenKind};
use super::super::ExpressionError;
use super::mapping;
use super::Adapter;
use crate::program::token::{Token as OuterToken, TokenKind as OuterKind};

impl Adapter<'_> {
    pub(super) fn scan(mut self) -> Result<Vec<Token>, ExpressionError> {
        while self.cursor < self.outer.len() {
            let token = self.outer[self.cursor].clone();
            if token.span.end <= self.position {
                self.cursor += 1;
                continue;
            }
            match token.kind {
                OuterKind::Word(_) | OuterKind::Number(_) => {
                    let start = token.span.start.max(self.position);
                    let consumed = self.fragment(start, token.span.end)?;
                    self.consume_through(consumed);
                }
                OuterKind::String(raw) => {
                    let start = token.span.start + 1;
                    let value = decode_string(&raw, start - self.origin)?;
                    self.push(TokenKind::Text(value), token.span.start, token.span.end);
                    self.consume_through(token.span.end);
                }
                OuterKind::Color(_) => self.color(token)?,
                OuterKind::Equals => {
                    let consumed = self.operators(token.span.start)?;
                    self.consume_through(consumed);
                }
                OuterKind::Eof => self.cursor += 1,
                OuterKind::LocalId(_) => return Err(self.unexpected('@', token.span.start)),
                OuterKind::DollarLeftBrace => return Err(self.unexpected('$', token.span.start)),
                kind => {
                    let mapped = mapping::simple(&kind).expect("special outer token handled above");
                    self.push(mapped, token.span.start, token.span.end);
                    self.consume_through(token.span.end);
                }
            }
        }
        let eof = self.end - self.origin;
        self.output.push(Token {
            kind: TokenKind::Eof,
            span: eof..eof,
        });
        Ok(self.output)
    }

    fn color(&mut self, token: OuterToken) -> Result<(), ExpressionError> {
        let bare_hash = token.span.end == token.span.start + 1;
        if bare_hash && self.next_is(&OuterKind::LeftBrace, token.span.end) {
            let end = self.outer[self.cursor + 1].span.end;
            self.push(TokenKind::MapStart, token.span.start, end);
            self.consume_through(end);
            return Ok(());
        }
        let mut end = token.span.end;
        while end < self.end && self.character(end).is_ascii_alphanumeric() {
            end += 1;
        }
        let span = self.local(token.span.start, end);
        let kind = validate_color(&self.source[token.span.start..end], span.clone())?;
        self.output.push(Token { kind, span });
        self.consume_through(end);
        Ok(())
    }

    fn next_is(&self, kind: &OuterKind, start: usize) -> bool {
        self.outer.get(self.cursor + 1).is_some_and(|token| {
            token.span.start == start
                && std::mem::discriminant(&token.kind) == std::mem::discriminant(kind)
        })
    }

    fn consume_through(&mut self, end: usize) {
        self.position = end;
        while self.cursor < self.outer.len() && self.outer[self.cursor].span.end <= end {
            self.cursor += 1;
        }
    }

    pub(super) fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.output.push(Token {
            kind,
            span: self.local(start, end),
        });
    }

    pub(super) fn local(&self, start: usize, end: usize) -> Range<usize> {
        start - self.origin..end - self.origin
    }

    pub(super) fn unexpected(&self, value: char, start: usize) -> ExpressionError {
        ExpressionError::new(
            "EXPRESSION_LEX_CHARACTER",
            format!("unexpected character `{value}`"),
            self.local(start, start + value.len_utf8()),
        )
    }
}
