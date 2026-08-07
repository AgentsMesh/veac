use super::Parser;
use crate::program::expression::ast::{LetBinding, MutableAssignment, MutableBinding, Statement};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::ExpressionError;
use crate::vocabulary::{accepts_expression_name, ExpressionNameKind};

impl Parser {
    pub(super) fn statement(&mut self) -> Result<Statement, ExpressionError> {
        let kind = self.advance();
        let start = kind.span.start;
        let (name, name_span) = self.local_name()?;
        if matches!(kind.kind, TokenKind::Set) {
            self.expect(&TokenKind::Equal, "=")?;
            let value = self.expression()?;
            let end = self.expect(&TokenKind::Semicolon, ";")?.end;
            return Ok(Statement::Set(MutableAssignment {
                name,
                name_span,
                value,
                span: start..end,
            }));
        }
        let annotation = self
            .take(&TokenKind::Colon)
            .map(|_| self.type_annotation())
            .transpose()?;
        self.expect(&TokenKind::Equal, "=")?;
        let value = self.expression()?;
        let end = self.expect(&TokenKind::Semicolon, ";")?.end;
        Ok(match kind.kind {
            TokenKind::Let => Statement::Let(LetBinding {
                name,
                name_span,
                annotation,
                value,
                span: start..end,
            }),
            TokenKind::Var => Statement::Var(MutableBinding {
                name,
                name_span,
                annotation,
                value,
                span: start..end,
            }),
            _ => unreachable!("statement starts with let, var, or set"),
        })
    }

    fn local_name(&mut self) -> Result<(String, std::ops::Range<usize>), ExpressionError> {
        let name = self.advance();
        let TokenKind::Symbol(value) = name.kind else {
            return Err(ExpressionError::new(
                "EXPRESSION_LOCAL_NAME",
                "expected a local binding name",
                name.span,
            ));
        };
        if !accepts_expression_name(&value, ExpressionNameKind::Local) {
            return Err(ExpressionError::new(
                "EXPRESSION_LOCAL_NAME",
                format!("`{value}` is not a valid local binding name"),
                name.span,
            ));
        }
        if self.at(&TokenKind::Dot) {
            return Err(ExpressionError::new(
                "EXPRESSION_LOCAL_NAME",
                "local binding names cannot be qualified",
                name.span.start..self.current().span.end,
            ));
        }
        Ok((value, name.span))
    }
}
