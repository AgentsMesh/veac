use super::{Diagnostic, Span};
mod cursor;
mod token;
pub(super) use token::{Token, TokenKind};

pub(super) fn lex(source: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    Lexer::new(source).scan()
}

struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn scan(mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        while let Some(character) = self.current() {
            if character.is_whitespace() {
                self.advance();
                continue;
            }
            if character == '/' && self.next() == Some('/') {
                self.line_comment();
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
                    self.diagnostics.push(Diagnostic {
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
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                start: self.offset,
                end: self.offset,
            },
        });
        (self.tokens, self.diagnostics)
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.offset;
        self.advance();
        self.tokens.push(Token {
            kind,
            span: Span {
                start,
                end: self.offset,
            },
        });
    }

    fn bare(&mut self, start: usize, number: bool) {
        while self.current().is_some_and(|character| {
            !character.is_whitespace() && !matches!(character, '{' | '}' | ';' | '"')
        }) {
            self.advance();
        }
        let raw = self.source[start..self.offset].to_owned();
        self.tokens.push(Token {
            kind: if number {
                TokenKind::Number(raw)
            } else {
                TokenKind::Word(raw)
            },
            span: Span {
                start,
                end: self.offset,
            },
        });
    }

    fn string(&mut self, start: usize) {
        self.advance();
        let mut value = String::new();
        let mut terminated = false;
        while let Some(character) = self.current() {
            if character == '"' {
                self.advance();
                terminated = true;
                break;
            }
            if character == '\\' {
                self.advance();
                let Some(escaped) = self.current() else { break };
                value.push(match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                });
                self.advance();
            } else {
                value.push(character);
                self.advance();
            }
        }
        let span = Span {
            start,
            end: self.offset,
        };
        if terminated {
            self.tokens.push(Token {
                kind: TokenKind::String(value),
                span,
            });
        } else {
            self.diagnostics.push(Diagnostic {
                code: "AUTHORING_LEX_STRING",
                message: "unterminated string literal".to_owned(),
                span,
            });
        }
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
            self.tokens.push(Token {
                kind: TokenKind::Color(raw.to_ascii_lowercase()),
                span: Span {
                    start,
                    end: self.offset,
                },
            });
        } else {
            self.diagnostics.push(Diagnostic {
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
