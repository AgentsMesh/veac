use super::{Diagnostic, Span};
mod cursor;
mod output;
mod string;
mod token;
pub(super) use token::{Token, TokenKind};

pub(super) fn lex_with_limits(
    source: &str,
    source_limit: usize,
    token_limit: usize,
) -> (Vec<Token>, Vec<Diagnostic>) {
    if source.len() > source_limit {
        return (
            vec![Token {
                kind: TokenKind::Eof,
                span: Span::default(),
            }],
            vec![Diagnostic {
                code: "AUTHORING_SOURCE_LIMIT",
                message: "authoring source exceeds 32 MiB".to_owned(),
                span: Span::default(),
            }],
        );
    }
    Lexer::new(source, token_limit).scan()
}

pub(super) const MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;
pub(super) const MAX_TOKENS: usize = 1_000_000;

struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    token_limit: usize,
    token_limit_reported: bool,
    diagnostic_limit_reported: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str, token_limit: usize) -> Self {
        Self {
            source,
            offset: 0,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
            token_limit,
            token_limit_reported: false,
            diagnostic_limit_reported: false,
        }
    }

    fn scan(mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        while let Some(character) = self.current() {
            if self.token_limit_reported || self.diagnostic_limit_reported {
                break;
            }
            if character.is_whitespace() {
                self.advance();
                continue;
            }
            if character == '/' && self.next() == Some('/') {
                self.line_comment();
                continue;
            }
            if character == '/' && self.next() == Some('*') {
                let start = self.offset;
                if !self.block_comment() {
                    self.report(Diagnostic {
                        code: "AUTHORING_LEX_BLOCK_COMMENT",
                        message: "unterminated block comment".to_owned(),
                        span: Span {
                            start,
                            end: self.offset,
                        },
                    });
                }
                continue;
            }
            let start = self.offset;
            match character {
                '{' => self.single(TokenKind::LeftBrace),
                '}' => self.single(TokenKind::RightBrace),
                ';' => self.single(TokenKind::Semicolon),
                '"' => self.string(start),
                '#' => self.color(start),
                value if value.is_ascii_digit() || self.starts_signed_number() => {
                    self.bare(start, true)
                }
                value if value.is_alphabetic() || matches!(value, '_' | '.') => {
                    self.bare(start, false)
                }
                _ => {
                    self.advance();
                    self.report(Diagnostic {
                        code: "AUTHORING_LEX_CHARACTER",
                        message: format!("unexpected character `{character}`"),
                        span: Span {
                            start,
                            end: self.offset,
                        },
                    });
                }
            }
        }
        self.push(TokenKind::Eof, self.offset, self.offset);
        (self.tokens, self.diagnostics)
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.offset;
        self.advance();
        self.push(kind, start, self.offset);
    }

    fn bare(&mut self, start: usize, number: bool) {
        while let Some(character) = self.current() {
            if character.is_whitespace()
                || matches!(character, '{' | '}' | ';' | '"')
                || character == '/' && matches!(self.next(), Some('/' | '*'))
            {
                break;
            }
            self.advance();
        }
        let raw = self.source[start..self.offset].to_owned();
        let kind = if number {
            TokenKind::Number(raw)
        } else {
            TokenKind::Word(raw)
        };
        self.push(kind, start, self.offset);
    }

    fn color(&mut self, start: usize) {
        self.advance();
        while self
            .current()
            .is_some_and(|value| value.is_ascii_hexdigit())
        {
            self.advance();
        }
        let raw = &self.source[start..self.offset];
        if matches!(raw.len(), 7 | 9) {
            self.push(
                TokenKind::Color(raw.to_ascii_lowercase()),
                start,
                self.offset,
            );
        } else {
            self.report(Diagnostic {
                code: "AUTHORING_LEX_COLOR",
                message: "color must be #rrggbb or #rrggbbaa".to_owned(),
                span: Span {
                    start,
                    end: self.offset,
                },
            });
        }
    }
}
