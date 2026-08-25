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
                    let decoded = decode_escape(&self.source[self.offset..], escape_start)?;
                    value.push(decoded.character);
                    let end = self.offset + decoded.bytes;
                    while self.offset < end {
                        self.advance();
                    }
                }
                Some(character) if character.is_control() => {
                    return Err(control_error(character, self.offset));
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
        self.tokens.push(Token {
            kind: validate_color(raw, start..self.offset)?,
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

pub(in crate::program::expression) fn decode_string(
    raw: &str,
    content_start: usize,
) -> Result<String, ExpressionError> {
    let mut value = String::new();
    let mut offset = 0;
    while let Some(character) = raw[offset..].chars().next() {
        if character == '\\' {
            let escape_start = offset;
            offset += character.len_utf8();
            if raw[offset..].chars().next().is_none() {
                return Err(ExpressionError::new(
                    "EXPRESSION_STRING_LITERAL",
                    "unterminated string literal",
                    content_start + escape_start..content_start + offset,
                ));
            }
            let decoded = decode_escape(&raw[offset..], content_start + escape_start)?;
            value.push(decoded.character);
            offset += decoded.bytes;
        } else if character.is_control() {
            return Err(control_error(character, content_start + offset));
        } else {
            value.push(character);
            offset += character.len_utf8();
        }
    }
    Ok(value)
}

fn decode_escape(
    raw: &str,
    escape_start: usize,
) -> Result<crate::string_codec::DecodedEscape, ExpressionError> {
    let current = raw
        .chars()
        .next()
        .expect("escape sequence starts with a character");
    crate::string_codec::decode_escape(raw).map_err(|message| {
        ExpressionError::new(
            "EXPRESSION_STRING_ESCAPE",
            message,
            escape_start..escape_start + 1 + current.len_utf8(),
        )
    })
}

fn control_error(character: char, start: usize) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_STRING_CONTROL",
        "control characters must use an escape sequence",
        start..start + character.len_utf8(),
    )
}

pub(in crate::program::expression) fn validate_color(
    raw: &str,
    span: std::ops::Range<usize>,
) -> Result<TokenKind, ExpressionError> {
    let valid =
        matches!(raw.len(), 7 | 9) && raw[1..].bytes().all(|value| value.is_ascii_hexdigit());
    valid
        .then(|| TokenKind::Color(raw.to_ascii_lowercase()))
        .ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_COLOR_LITERAL",
                "color must be #rrggbb or #rrggbbaa",
                span,
            )
        })
}
