mod comment;
mod lexeme;
mod literal;
mod operator;
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

#[cfg(test)]
mod tests;

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
                '-' if self.next() == Some('>') => self.pair(TokenKind::Arrow),
                '-' => self.single(TokenKind::Minus),
                '*' => self.single(TokenKind::Star),
                '/' => self.single(TokenKind::Slash),
                '!' => self.one_or_two(TokenKind::Bang, '=', TokenKind::BangEqual),
                '=' if self.next() == Some('>') => self.pair(TokenKind::FatArrow),
                '=' => self.one_or_two(TokenKind::Equal, '=', TokenKind::EqualEqual),
                '<' => self.one_or_two(TokenKind::Less, '=', TokenKind::LessEqual),
                '>' => self.one_or_two(TokenKind::Greater, '=', TokenKind::GreaterEqual),
                '&' => self.required_pair('&', TokenKind::AndAnd)?,
                '|' => self.required_pair('|', TokenKind::OrOr)?,
                '(' => self.single(TokenKind::LeftParen),
                ')' => self.single(TokenKind::RightParen),
                '[' => self.single(TokenKind::LeftBracket),
                ']' => self.single(TokenKind::RightBracket),
                '{' => self.single(TokenKind::LeftBrace),
                '}' => self.single(TokenKind::RightBrace),
                '#' if self.next() == Some('{') => self.pair(TokenKind::MapStart),
                ':' => self.single(TokenKind::Colon),
                ';' => self.single(TokenKind::Semicolon),
                ',' => self.single(TokenKind::Comma),
                '"' => self.string(start)?,
                '#' => self.color(start)?,
                '.' if self.next() == Some('.') => self.pair(TokenKind::DotDot),
                value if value.is_ascii_digit() || value == '.' && self.next_is_digit() => {
                    self.number(start)?
                }
                '.' => self.single(TokenKind::Dot),
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
