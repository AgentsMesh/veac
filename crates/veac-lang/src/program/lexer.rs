mod cursor;
mod output;

use crate::authoring::Span;

use super::diagnostic::Diagnostic;
use super::limits::{check_source_size, MAX_SOURCE_TOKENS};
use super::token::{is_word_continue, is_word_start, Token, TokenKind};

pub(crate) fn lex(path: &str, source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    check_source_size(path, source, Span::default()).map_err(|error| vec![error])?;
    lex_with_limit(path, source, MAX_SOURCE_TOKENS)
}

pub(crate) fn lex_with_limit(
    path: &str,
    source: &str,
    token_limit: usize,
) -> Result<Vec<Token>, Vec<Diagnostic>> {
    Lexer::new(path, source, token_limit).scan()
}

struct Lexer<'a> {
    path: &'a str,
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    token_limit: usize,
    token_limit_reported: bool,
    diagnostic_limit_reported: bool,
}

impl<'a> Lexer<'a> {
    fn new(path: &'a str, source: &'a str, token_limit: usize) -> Self {
        Self {
            path,
            source,
            offset: 0,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
            token_limit,
            token_limit_reported: false,
            diagnostic_limit_reported: false,
        }
    }

    fn scan(mut self) -> Result<Vec<Token>, Vec<Diagnostic>> {
        while let Some(value) = self.current() {
            if self.token_limit_reported || self.diagnostic_limit_reported {
                break;
            }
            if value.is_whitespace() {
                self.advance();
            } else if value == '/' && self.next() == Some('/') {
                self.line_comment();
            } else if value == '/' && self.next() == Some('*') {
                self.block_comment();
            } else {
                self.token(value);
            }
        }
        self.push(TokenKind::Eof, self.offset, self.offset);
        if self.diagnostics.is_empty() {
            Ok(self.tokens)
        } else {
            Err(self.diagnostics)
        }
    }

    fn token(&mut self, value: char) {
        let start = self.offset;
        match value {
            '{' => self.single(TokenKind::LeftBrace),
            '}' => self.single(TokenKind::RightBrace),
            '(' => self.single(TokenKind::LeftParen),
            ')' => self.single(TokenKind::RightParen),
            ';' => self.single(TokenKind::Semicolon),
            ',' => self.single(TokenKind::Comma),
            '=' => self.single(TokenKind::Equals),
            '+' => self.single(TokenKind::Plus),
            '-' => self.single(TokenKind::Minus),
            '*' => self.single(TokenKind::Star),
            '/' => self.single(TokenKind::Slash),
            '$' if self.next() == Some('{') => self.dollar_brace(),
            '"' => self.string(start),
            '#' => self.color(start),
            '@' => self.local_id(start),
            value if value.is_ascii_digit() || value == '.' => self.number(start),
            value if is_word_start(value) => self.word(start),
            _ => self.invalid(value, start),
        }
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.offset;
        self.advance();
        self.push(kind, start, self.offset);
    }

    fn dollar_brace(&mut self) {
        let start = self.offset;
        self.advance();
        self.advance();
        self.push(TokenKind::DollarLeftBrace, start, self.offset);
    }

    fn word(&mut self, start: usize) {
        self.take_while(is_word_continue);
        self.push(
            TokenKind::Word(self.source[start..self.offset].to_owned()),
            start,
            self.offset,
        );
    }

    fn number(&mut self, start: usize) {
        self.take_while(|value| value.is_alphanumeric() || matches!(value, '.' | '%'));
        self.push(
            TokenKind::Number(self.source[start..self.offset].to_owned()),
            start,
            self.offset,
        );
    }

    fn local_id(&mut self, start: usize) {
        self.advance();
        let value_start = self.offset;
        self.take_while(is_word_continue);
        let value = self.source[value_start..self.offset].to_owned();
        if value.is_empty() {
            self.invalid('@', start);
        } else {
            self.push(TokenKind::LocalId(value), start, self.offset);
        }
    }

    fn string(&mut self, start: usize) {
        self.advance();
        let mut escaped = false;
        while let Some(value) = self.current() {
            self.advance();
            if value == '"' && !escaped {
                let raw = &self.source[start + 1..self.offset - 1];
                self.push(TokenKind::String(raw.to_owned()), start, self.offset);
                return;
            }
            escaped = value == '\\' && !escaped;
            if value != '\\' {
                escaped = false;
            }
        }
        self.invalid('"', start);
    }

    fn color(&mut self, start: usize) {
        self.advance();
        self.take_while(|value| value.is_ascii_hexdigit());
        self.push(
            TokenKind::Color(self.source[start..self.offset].to_owned()),
            start,
            self.offset,
        );
    }

    fn line_comment(&mut self) {
        self.take_while(|value| value != '\n');
    }

    fn block_comment(&mut self) {
        let start = self.offset;
        self.advance();
        self.advance();
        while self.current().is_some() {
            if self.current() == Some('*') && self.next() == Some('/') {
                self.advance();
                self.advance();
                return;
            }
            self.advance();
        }
        self.invalid('/', start);
    }
}

#[cfg(test)]
#[path = "lexer/tests.rs"]
mod tests;
