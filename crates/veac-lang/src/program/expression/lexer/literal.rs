use super::{Lexer, Token, TokenKind};
use crate::program::expression::ExpressionError;

impl Lexer<'_> {
    pub(super) fn string(&mut self, start: usize) -> Result<(), ExpressionError> {
        self.advance();
        let mut value = String::new();
        loop {
            match self.current() {
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    let escape_start = self.offset;
                    self.advance();
                    if self.current().is_none() {
                        return Err(self.string_error(start));
                    }
                    let decoded = crate::string_codec::decode_escape(&self.source[self.offset..])
                        .map_err(|message| {
                        ExpressionError::new(
                            "EXPRESSION_STRING_ESCAPE",
                            message,
                            escape_start..self.offset + self.current().map_or(0, char::len_utf8),
                        )
                    })?;
                    value.push(decoded.character);
                    let end = self.offset + decoded.bytes;
                    while self.offset < end {
                        self.advance();
                    }
                }
                Some(character) if character.is_control() => {
                    return Err(ExpressionError::new(
                        "EXPRESSION_STRING_CONTROL",
                        "control characters must use an escape sequence",
                        self.offset..self.offset + character.len_utf8(),
                    ))
                }
                Some(character) => {
                    value.push(character);
                    self.advance();
                }
                None => return Err(self.string_error(start)),
            }
        }
        self.tokens.push(Token {
            kind: TokenKind::Text(value),
            span: start..self.offset,
        });
        Ok(())
    }

    pub(super) fn color(&mut self, start: usize) -> Result<(), ExpressionError> {
        self.advance();
        while self
            .current()
            .is_some_and(|value| value.is_ascii_alphanumeric())
        {
            self.advance();
        }
        let raw = &self.source[start..self.offset];
        let valid =
            matches!(raw.len(), 7 | 9) && raw[1..].bytes().all(|value| value.is_ascii_hexdigit());
        if !valid {
            return Err(ExpressionError::new(
                "EXPRESSION_COLOR_LITERAL",
                "color must be #rrggbb or #rrggbbaa",
                start..self.offset,
            ));
        }
        self.tokens.push(Token {
            kind: TokenKind::Color(raw.to_ascii_lowercase()),
            span: start..self.offset,
        });
        Ok(())
    }

    fn string_error(&self, start: usize) -> ExpressionError {
        ExpressionError::new(
            "EXPRESSION_STRING_LITERAL",
            "unterminated string literal",
            start..self.offset,
        )
    }
}
