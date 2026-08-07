use super::{Lexer, Token, TokenKind};
use crate::program::expression::ExpressionError;

impl Lexer<'_> {
    pub(super) fn number(&mut self, start: usize) -> Result<(), ExpressionError> {
        let mut decimal = false;
        while let Some(character) = self.current() {
            if character.is_ascii_digit() {
                self.advance();
            } else if character == '.' && self.next() == Some('.') {
                break;
            } else if character == '.'
                && self.next().is_some_and(|value| value.is_ascii_digit())
                && !decimal
            {
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
        if self.current() == Some('_') {
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

    pub(super) fn symbol(&mut self, start: usize) -> Result<(), ExpressionError> {
        self.symbol_segment();
        let value = &self.source[start..self.offset];
        let kind = classify_symbol(value).ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_SYMBOL",
                format!("symbol segments must be {}", crate::name::NAME_CONTRACT),
                start..self.offset,
            )
        })?;
        self.tokens.push(Token {
            kind,
            span: start..self.offset,
        });
        Ok(())
    }

    fn symbol_segment(&mut self) {
        while self.current().is_some_and(|character| {
            character.is_ascii_alphanumeric()
                || character == '_'
                || character == '-'
                    && self
                        .next()
                        .is_some_and(|next| next.is_ascii_alphanumeric() || next == '_')
        }) {
            self.advance();
        }
    }
}

fn classify_symbol(value: &str) -> Option<TokenKind> {
    use crate::vocabulary::control_uses::expression as controls;
    if let Some(literal) = crate::vocabulary::ReservedLiteral::parse(value) {
        Some(TokenKind::Bool(literal.value()))
    } else if controls::LET_BINDING.matches(value) {
        Some(TokenKind::Let)
    } else if controls::MUTABLE_BINDING.matches(value) {
        Some(TokenKind::Var)
    } else if controls::MUTABLE_ASSIGNMENT.matches(value) {
        Some(TokenKind::Set)
    } else if controls::IF_BRANCH.matches(value) {
        Some(TokenKind::If)
    } else if controls::ELSE_BRANCH.matches(value) {
        Some(TokenKind::Else)
    } else if controls::RANGE_STEP.matches(value) {
        Some(TokenKind::By)
    } else if controls::CLOSURE.matches(value) {
        Some(TokenKind::Fn)
    } else if controls::FOR_BINDING.matches(value) {
        Some(TokenKind::For)
    } else if controls::FOR_SOURCE.matches(value) {
        Some(TokenKind::In)
    } else if controls::MATCH_BRANCH.matches(value) {
        Some(TokenKind::Match)
    } else if controls::TEMPORAL_ATTACHMENT.matches(value) {
        Some(TokenKind::Animate)
    } else if crate::name::is_name(value) {
        Some(TokenKind::Symbol(value.to_owned()))
    } else {
        None
    }
}
