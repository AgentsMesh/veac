mod declaration;
mod file;
mod function;
mod input;
pub(crate) mod kind;
mod method;
mod name;
mod slice;
mod temporal;
mod type_declaration;

use crate::authoring::Span;
use crate::vocabulary::ControlUse;

use super::diagnostic::Diagnostic;
use super::syntax_document::{SyntaxDocument, SyntaxSlice};
use super::token::{Token, TokenKind};

pub(crate) use file::{parse, parse_declaration_fragment, parse_executable};
pub(crate) use type_declaration::{validate_fragment, TypeDeclarationFragmentKind};

pub(super) struct Parser<'a> {
    path: &'a str,
    document: SyntaxDocument,
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn new(path: &'a str, document: SyntaxDocument) -> Self {
        Self {
            path,
            document,
            cursor: 0,
        }
    }

    pub(super) fn at_control(&self, expected: ControlUse) -> bool {
        self.current()
            .word()
            .is_some_and(|word| expected.matches(word))
    }

    pub(super) fn at(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current().kind) == std::mem::discriminant(kind)
    }

    pub(super) fn at_eof(&self) -> bool {
        self.at(&TokenKind::Eof)
    }

    pub(super) fn current(&self) -> &Token {
        &self.document.tokens()[self.cursor]
    }

    pub(super) const fn mark(&self) -> usize {
        self.cursor
    }

    pub(super) fn slice_from(&self, start: usize, span: Span) -> SyntaxSlice {
        SyntaxSlice {
            span,
            tokens: start..self.cursor,
        }
    }

    pub(super) fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !self.at_eof() {
            self.cursor += 1;
        }
        token
    }

    pub(super) fn word(&mut self, context: &str) -> Result<(String, Span), Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::Word(value) => Ok((value, token.span)),
            _ => Err(self.error(
                "PROGRAM_EXPECTED_WORD",
                format!("expected {context}"),
                token.span,
            )),
        }
    }

    pub(super) fn local_id(&mut self, context: &str) -> Result<(String, Span), Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::LocalId(value) if crate::name::is_name(&value) => Ok((value, token.span)),
            TokenKind::LocalId(_) => Err(self.error(
                "PROGRAM_IDENTIFIER",
                format!("{context} must be {}", crate::name::NAME_CONTRACT),
                token.span,
            )),
            _ => Err(self.error(
                "PROGRAM_EXPECTED_LOCAL_ID",
                format!("expected @{context}"),
                token.span,
            )),
        }
    }

    pub(super) fn string(&mut self, context: &str) -> Result<(String, Span), Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::String(value) => Ok((value, token.span)),
            _ => Err(self.error(
                "PROGRAM_EXPECTED_STRING",
                format!("expected {context}"),
                token.span,
            )),
        }
    }

    pub(super) fn expect_control(&mut self, expected: ControlUse) -> Result<Span, Diagnostic> {
        if self.at_control(expected) {
            Ok(self.advance().span)
        } else {
            Err(self.error(
                "PROGRAM_EXPECTED_WORD",
                format!("expected `{}`", expected.as_str()),
                self.current().span,
            ))
        }
    }

    pub(super) fn expect(&mut self, kind: TokenKind, label: &str) -> Result<Span, Diagnostic> {
        if self.at(&kind) {
            Ok(self.advance().span)
        } else {
            Err(self.error(
                "PROGRAM_EXPECTED_TOKEN",
                format!("expected {label}"),
                self.current().span,
            ))
        }
    }

    pub(super) fn error(
        &self,
        code: &'static str,
        message: impl Into<String>,
        span: Span,
    ) -> Diagnostic {
        Diagnostic::new(code, self.path, message, span)
    }
}
