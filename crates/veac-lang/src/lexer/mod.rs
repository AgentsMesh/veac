mod scan;

use crate::error::{ErrorKind, VeacError};
use crate::token::{Span, Token, TokenKind};

/// Lexical analyzer for VEAC source code.
pub struct Lexer<'a> {
    pub(crate) _source: &'a str,
    pub(crate) chars: Vec<char>,
    pub(crate) pos: usize,
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            _source: source,
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Tokenize the entire source, returning all tokens or the first error.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, VeacError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    pub(crate) fn current(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    pub(crate) fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    pub(crate) fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub(crate) fn span_from(&self, start: usize, start_line: usize, start_col: usize) -> Span {
        Span::new(start, self.pos, start_line, start_col)
    }

    pub(crate) fn next_token(&mut self) -> Result<Token, VeacError> {
        self.skip_whitespace();

        let start = self.pos;
        let start_line = self.line;
        let start_col = self.col;

        let ch = match self.current() {
            Some(ch) => ch,
            None => {
                return Ok(Token::new(
                    TokenKind::Eof,
                    self.span_from(start, start_line, start_col),
                ));
            }
        };

        if ch == '/' && self.peek() == Some('/') {
            return self.lex_line_comment();
        }
        if ch == '"' {
            return self.lex_string(start, start_line, start_col);
        }
        if ch == '#' {
            return self.lex_color(start, start_line, start_col);
        }
        if ch.is_ascii_digit() {
            return self.lex_number(start, start_line, start_col);
        }
        if ch == '-' && self.peek().is_some_and(|c| c.is_ascii_digit()) {
            return self.lex_negative_number(start, start_line, start_col);
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            return self.lex_ident_or_keyword(start, start_line, start_col);
        }

        let kind = match ch {
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '=' => TokenKind::Equals,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            _ => {
                return Err(VeacError::new(
                    ErrorKind::UnexpectedChar,
                    format!("unexpected character `{ch}`"),
                    Some(self.span_from(start, start_line, start_col)),
                ));
            }
        };

        self.advance();
        Ok(Token::new(
            kind,
            self.span_from(start, start_line, start_col),
        ))
    }
}

/// Check if a string matches SMPTE timecode format: HH:MM:SS:FF
pub(crate) fn is_smpte(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_digit()))
}
