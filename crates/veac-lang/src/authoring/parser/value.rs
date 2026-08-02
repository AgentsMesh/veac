use super::Parser;
use crate::authoring::lexer::TokenKind;
use crate::authoring::{Identifier, NumberLiteral, Spanned};

impl Parser {
    pub(super) fn identifier(&mut self, context: &'static str) -> Option<Identifier> {
        let token = self.advance();
        if let TokenKind::Word(value) = token.kind {
            Some(Spanned {
                value,
                span: token.span,
            })
        } else {
            self.error(
                "AUTHORING_EXPECTED_IDENTIFIER",
                format!("expected {context} identifier"),
                token.span,
            );
            None
        }
    }

    pub(super) fn stable_identifier(&mut self, context: &'static str) -> Option<Identifier> {
        let value = self.identifier(context)?;
        if crate::name::is_name(&value.value) {
            Some(value)
        } else {
            self.error(
                "AUTHORING_IDENTIFIER",
                format!("{context} must be {}", crate::name::NAME_CONTRACT),
                value.span,
            );
            None
        }
    }

    pub(super) fn number(&mut self, context: &'static str) -> Option<NumberLiteral> {
        let token = self.advance();
        if let TokenKind::Number(raw) = token.kind {
            Some(NumberLiteral {
                raw,
                span: token.span,
            })
        } else {
            self.error(
                "AUTHORING_EXPECTED_NUMBER",
                format!("expected numeric {context}"),
                token.span,
            );
            None
        }
    }

    pub(super) fn color(&mut self, context: &'static str) -> Option<Spanned<String>> {
        let token = self.advance();
        if let TokenKind::Color(value) = token.kind {
            Some(Spanned {
                value,
                span: token.span,
            })
        } else {
            self.error(
                "AUTHORING_EXPECTED_COLOR",
                format!("expected {context} color"),
                token.span,
            );
            None
        }
    }
}
