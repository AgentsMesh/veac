use super::super::lexer::{classify_symbol, TokenKind};
use super::super::ExpressionError;
use super::Adapter;

impl Adapter<'_> {
    pub(super) fn fragment(
        &mut self,
        mut at: usize,
        boundary: usize,
    ) -> Result<usize, ExpressionError> {
        while at < boundary {
            let character = self.character(at);
            if character.is_ascii_digit()
                || character == '.' && self.next_character(at).is_some_and(|v| v.is_ascii_digit())
            {
                at = self.number(at, boundary)?;
            } else if character == '.' {
                self.push(TokenKind::Dot, at, at + 1);
                at += 1;
            } else if crate::name::is_name_start(character) {
                at = self.symbol(at)?;
            } else if character == '-' {
                if self.next_character(at) == Some('>') {
                    at = self.operators(at)?;
                } else {
                    self.push(TokenKind::Minus, at, at + 1);
                    at += 1;
                }
            } else {
                return Err(self.unexpected(character, at));
            }
        }
        Ok(at)
    }

    fn number(&mut self, start: usize, boundary: usize) -> Result<usize, ExpressionError> {
        let mut at = start;
        let mut decimal = false;
        while at < boundary {
            let value = self.character(at);
            if value.is_ascii_digit() {
                at += 1;
            } else if value == '.' && self.next_character(at) == Some('.') {
                break;
            } else if value == '.'
                && self.next_character(at).is_some_and(|v| v.is_ascii_digit())
                && !decimal
            {
                decimal = true;
                at += 1;
            } else {
                break;
            }
        }
        let number_end = at;
        while at < boundary && self.character(at).is_ascii_alphabetic() {
            at += 1;
        }
        if at < boundary && self.character(at) == '%' {
            at += 1;
        }
        if at < self.end && self.character(at) == '_' {
            return Err(ExpressionError::new(
                "EXPRESSION_NUMBER_LITERAL",
                "invalid numeric literal",
                self.local(start, at + 1),
            ));
        }
        self.push(
            TokenKind::Number {
                number: self.source[start..number_end].to_owned(),
                unit: self.source[number_end..at].to_ascii_lowercase(),
            },
            start,
            at,
        );
        Ok(at)
    }

    fn symbol(&mut self, start: usize) -> Result<usize, ExpressionError> {
        let mut at = start;
        while at < self.end {
            let value = self.character(at);
            let next = self.next_character(at);
            if value.is_ascii_alphanumeric()
                || value == '_'
                || value == '-' && next.is_some_and(|v| v.is_ascii_alphanumeric() || v == '_')
            {
                at += value.len_utf8();
            } else {
                break;
            }
        }
        let value = &self.source[start..at];
        let kind = classify_symbol(value).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_SYMBOL",
                format!("symbol segments must be {}", crate::name::NAME_CONTRACT),
                self.local(start, at),
            )
        })?;
        self.push(kind, start, at);
        Ok(at)
    }

    pub(super) fn character(&self, at: usize) -> char {
        self.source[at..]
            .chars()
            .next()
            .expect("token span contains a character")
    }

    pub(super) fn next_character(&self, at: usize) -> Option<char> {
        let current = self.character(at);
        (at + current.len_utf8() < self.end).then(|| self.character(at + current.len_utf8()))
    }
}
