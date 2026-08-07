use super::{Parser, ValueTypeError, ValueTypeParseError, MAX_VALUE_TYPE_ARITY};

impl<'a> Parser<'a> {
    pub(super) fn word(&mut self) -> Result<&'a str, ValueTypeParseError> {
        let start = self.cursor;
        while self.byte().is_some_and(|value| value.is_ascii_alphabetic()) {
            self.cursor += 1;
        }
        (start != self.cursor)
            .then_some(&self.source[start..self.cursor])
            .ok_or_else(|| self.error("expected value type"))
    }

    pub(super) fn expect(&mut self, value: char) -> Result<(), ValueTypeParseError> {
        self.take(value)
            .then_some(())
            .ok_or_else(|| self.error(format!("expected `{value}`")))
    }

    pub(super) fn take(&mut self, value: char) -> bool {
        self.whitespace();
        if self.byte() == Some(value as u8) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn peek(&mut self, value: char) -> bool {
        self.whitespace();
        self.byte() == Some(value as u8)
    }

    pub(super) fn whitespace(&mut self) {
        while self.byte().is_some_and(|value| value.is_ascii_whitespace()) {
            self.cursor += 1;
        }
    }

    fn byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.cursor).copied()
    }

    pub(super) fn check_arity(
        &self,
        arity: usize,
        start: usize,
    ) -> Result<(), ValueTypeParseError> {
        (arity <= MAX_VALUE_TYPE_ARITY)
            .then_some(())
            .ok_or_else(|| ValueTypeParseError {
                code: "VALUE_TYPE_ARITY",
                message: format!("type exceeds the {MAX_VALUE_TYPE_ARITY} element arity limit"),
                span: start..self.cursor,
            })
    }

    pub(super) fn construct(&self, error: ValueTypeError, start: usize) -> ValueTypeParseError {
        ValueTypeParseError::construction(error, start..self.cursor)
    }

    pub(super) fn error(&self, message: impl Into<String>) -> ValueTypeParseError {
        ValueTypeParseError::syntax(message, self.cursor..self.cursor.saturating_add(1))
    }
}
