mod comment;
mod literal;
mod token;

pub(super) use token::{Token, TokenKind};

use super::ExpressionError;

pub(super) fn lex(source: &str) -> Result<Vec<Token>, ExpressionError> {
    Lexer::new(source).scan()
}

struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            tokens: Vec::new(),
        }
    }

    fn scan(mut self) -> Result<Vec<Token>, ExpressionError> {
        while let Some(character) = self.current() {
            if character.is_whitespace() {
                self.advance();
                continue;
            }
            if character == '/' && self.next() == Some('/') {
                self.line_comment();
                continue;
            }
            if character == '/' && self.next() == Some('*') {
                self.block_comment()?;
                continue;
            }
            let start = self.offset;
            match character {
                '+' => self.single(TokenKind::Plus),
                '-' => self.single(TokenKind::Minus),
                '*' => self.single(TokenKind::Star),
                '/' => self.single(TokenKind::Slash),
                '(' => self.single(TokenKind::LeftParen),
                ')' => self.single(TokenKind::RightParen),
                ',' => self.single(TokenKind::Comma),
                '"' => self.string(start)?,
                '#' => self.color(start)?,
                value if value.is_ascii_digit() || value == '.' && self.next_is_digit() => {
                    self.number(start)?
                }
                value if crate::name::is_name_start(value) => self.symbol(start)?,
                _ => {
                    self.advance();
                    return Err(ExpressionError::new(
                        "EXPRESSION_LEX_CHARACTER",
                        format!("unexpected character `{character}`"),
                        start..self.offset,
                    ));
                }
            }
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: self.offset..self.offset,
        });
        Ok(self.tokens)
    }

    fn single(&mut self, kind: TokenKind) {
        let start = self.offset;
        self.advance();
        self.tokens.push(Token {
            kind,
            span: start..self.offset,
        });
    }

    fn number(&mut self, start: usize) -> Result<(), ExpressionError> {
        let mut decimal = false;
        while let Some(character) = self.current() {
            if character.is_ascii_digit() {
                self.advance();
            } else if character == '.' && !decimal {
                decimal = true;
                self.advance();
            } else {
                break;
            }
        }
        let number_end = self.offset;
        while self
            .current()
            .is_some_and(|value| value.is_ascii_alphabetic())
        {
            self.advance();
        }
        if self.current() == Some('%') {
            self.advance();
        }
        if self
            .current()
            .is_some_and(|value| value == '.' || value == '_')
        {
            return Err(ExpressionError::new(
                "EXPRESSION_NUMBER_LITERAL",
                "invalid numeric literal",
                start..self.offset + self.current().map_or(0, char::len_utf8),
            ));
        }
        self.tokens.push(Token {
            kind: TokenKind::Number {
                number: self.source[start..number_end].to_owned(),
                unit: self.source[number_end..self.offset].to_ascii_lowercase(),
            },
            span: start..self.offset,
        });
        Ok(())
    }

    fn symbol(&mut self, start: usize) -> Result<(), ExpressionError> {
        self.symbol_segment();
        while self.current() == Some('.') {
            self.advance();
            if !self.current().is_some_and(crate::name::is_name_start) {
                return Err(ExpressionError::new(
                    "EXPRESSION_SYMBOL",
                    "qualified symbol requires a segment after `.`",
                    start..self.offset,
                ));
            }
            self.symbol_segment();
        }
        let value = &self.source[start..self.offset];
        let kind = match value {
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ if crate::name::is_qualified_name(value) => TokenKind::Symbol(value.to_owned()),
            _ => {
                return Err(ExpressionError::new(
                    "EXPRESSION_SYMBOL",
                    format!("symbol segments must be {}", crate::name::NAME_CONTRACT),
                    start..self.offset,
                ))
            }
        };
        self.tokens.push(Token {
            kind,
            span: start..self.offset,
        });
        Ok(())
    }

    fn symbol_segment(&mut self) {
        while let Some(character) = self.current() {
            if character.is_ascii_alphanumeric()
                || character == '_'
                || character == '-'
                    && self
                        .next()
                        .is_some_and(|next| next.is_ascii_alphanumeric() || next == '_')
            {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub(super) fn current(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn next_is_digit(&self) -> bool {
        self.next().is_some_and(|value| value.is_ascii_digit())
    }

    pub(super) fn next(&self) -> Option<char> {
        self.source[self.offset..].chars().nth(1)
    }

    pub(super) fn advance(&mut self) {
        self.offset += self.current().map_or(0, char::len_utf8);
    }
}
