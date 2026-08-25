use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::model::RawBlock;
use crate::program::syntax_document::SyntaxSlice;
use crate::program::token::TokenKind;

use super::Parser;

impl Parser<'_> {
    pub(super) fn expression_until_parameter_end(
        &mut self,
    ) -> Result<(SyntaxSlice, Span), Diagnostic> {
        let token_start = self.mark();
        let start = self.current().span.start;
        let mut parentheses = 0usize;
        let mut braces = 0usize;
        let mut brackets = 0usize;
        while !self.at_eof() {
            match self.current().kind {
                TokenKind::LeftParen => parentheses += 1,
                TokenKind::RightParen if parentheses > 0 => parentheses -= 1,
                TokenKind::LeftBrace | TokenKind::DollarLeftBrace => braces += 1,
                TokenKind::RightBrace if braces > 0 => braces -= 1,
                TokenKind::LeftBracket => brackets += 1,
                TokenKind::RightBracket if brackets > 0 => brackets -= 1,
                TokenKind::Comma | TokenKind::RightParen
                    if parentheses == 0 && braces == 0 && brackets == 0 =>
                {
                    break
                }
                _ => {}
            }
            self.advance();
        }
        let tokens = token_start..self.mark();
        let Some(span) = self.document.token_span(tokens.clone()) else {
            return Err(self.error(
                "PROGRAM_EMPTY_PARAMETER_DEFAULT",
                "parameter default expression cannot be empty",
                Span { start, end: start },
            ));
        };
        Ok((SyntaxSlice { span, tokens }, span))
    }

    pub(super) fn expression_until_semicolon(&mut self) -> Result<(SyntaxSlice, Span), Diagnostic> {
        let token_start = self.mark();
        let start = self.current().span.start;
        let mut parentheses = 0usize;
        let mut braces = 0usize;
        while !self.at_eof() {
            match self.current().kind {
                TokenKind::LeftParen => parentheses += 1,
                TokenKind::RightParen if parentheses > 0 => parentheses -= 1,
                TokenKind::LeftBrace | TokenKind::DollarLeftBrace => braces += 1,
                TokenKind::RightBrace if braces > 0 => braces -= 1,
                TokenKind::Semicolon if parentheses == 0 && braces == 0 => break,
                _ => {}
            }
            self.advance();
        }
        let end = self.current().span.start;
        let token_end = self.mark();
        let tokens = token_start..token_end;
        let Some(span) = self.document.token_span(tokens.clone()) else {
            return Err(self.error(
                "PROGRAM_EMPTY_EXPRESSION",
                "expression cannot be empty",
                Span { start, end },
            ));
        };
        let expression = SyntaxSlice { span, tokens };
        let terminator = self.expect(TokenKind::Semicolon, "`;`")?;
        Ok((expression, terminator))
    }

    pub(super) fn raw_block(&mut self) -> Result<RawBlock, Diagnostic> {
        let start = self.mark();
        let left = self.expect(TokenKind::LeftBrace, "`{`")?;
        let mut depth = 1usize;
        while depth > 0 && !self.at_eof() {
            let token = self.advance();
            match token.kind {
                TokenKind::LeftBrace | TokenKind::DollarLeftBrace => depth += 1,
                TokenKind::RightBrace => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                let span = left.join(token.span);
                return Ok(RawBlock {
                    span,
                    syntax: self.slice_from(start, span),
                });
            }
        }
        Err(self.error("PROGRAM_UNCLOSED_BLOCK", "block is not closed", left))
    }
}
