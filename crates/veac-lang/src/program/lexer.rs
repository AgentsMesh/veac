mod cursor;
mod number;
mod operator;
mod output;
mod trivia;

use crate::authoring::Span;

use super::diagnostic::Diagnostic;
use super::limits::{check_source_size, MAX_SOURCE_TOKENS};
use super::syntax_document::{SyntaxDocument, SyntaxElement, SyntaxElementKind};
use super::token::{is_word_continue, is_word_start, Token, TokenKind};

pub(crate) fn lex(path: &str, source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    lex_document(path, source).map(|document| document.tokens().to_vec())
}

pub(crate) fn lex_document(path: &str, source: &str) -> Result<SyntaxDocument, Vec<Diagnostic>> {
    check_source_size(path, source, Span::default()).map_err(|error| vec![error])?;
    lex_document_with_limit(path, source, MAX_SOURCE_TOKENS)
}

fn lex_document_with_limit(
    path: &str,
    source: &str,
    token_limit: usize,
) -> Result<SyntaxDocument, Vec<Diagnostic>> {
    Lexer::new(path, source, token_limit).scan()
}

struct Lexer<'a> {
    path: &'a str,
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
    elements: Vec<SyntaxElement>,
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
            elements: Vec::new(),
            diagnostics: Vec::new(),
            token_limit,
            token_limit_reported: false,
            diagnostic_limit_reported: false,
        }
    }

    fn scan(mut self) -> Result<SyntaxDocument, Vec<Diagnostic>> {
        while let Some(value) = self.current() {
            if self.token_limit_reported || self.diagnostic_limit_reported {
                break;
            }
            if value.is_whitespace() {
                self.whitespace();
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
            Ok(SyntaxDocument::new(self.source, self.tokens, self.elements))
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
            '[' => self.single(TokenKind::LeftBracket),
            ']' => self.single(TokenKind::RightBracket),
            ':' => self.single(TokenKind::Colon),
            ';' => self.single(TokenKind::Semicolon),
            ',' => self.single(TokenKind::Comma),
            '=' => self.one_or_two(TokenKind::Equals, '=', TokenKind::EqualsEquals),
            '!' => self.one_or_two(TokenKind::Bang, '=', TokenKind::BangEquals),
            '<' => self.one_or_two(TokenKind::Less, '=', TokenKind::LessEquals),
            '>' => self.one_or_two(TokenKind::Greater, '=', TokenKind::GreaterEquals),
            '&' => self.required_pair('&', TokenKind::AndAnd),
            '|' => self.required_pair('|', TokenKind::OrOr),
            '+' => self.single(TokenKind::Plus),
            '-' if self.next() == Some('>') => self.arrow(),
            '-' => self.single(TokenKind::Minus),
            '*' => self.single(TokenKind::Star),
            '/' => self.single(TokenKind::Slash),
            '$' if self.next() == Some('{') => self.dollar_brace(),
            '"' => self.string(start),
            '#' => self.color(start),
            '@' => self.local_id(start),
            '.' if self.next() == Some('.') => self.required_pair('.', TokenKind::DotDot),
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
        while self.current().is_some_and(is_word_continue)
            && !(self.current() == Some('.') && self.next() == Some('.'))
        {
            self.advance();
        }
        self.push(
            TokenKind::Word(self.source[start..self.offset].to_owned()),
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
}

#[cfg(test)]
#[path = "lexer/tests.rs"]
mod tests;
