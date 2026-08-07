use std::collections::BTreeSet;

use super::Parser;
use crate::program::expression::ast::{Block, Expression, ExpressionKind, Statement};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::ExpressionError;

impl Parser {
    pub(super) fn block(
        &mut self,
        opening: std::ops::Range<usize>,
    ) -> Result<(Block, std::ops::Range<usize>), ExpressionError> {
        self.enter_depth(opening.clone())?;
        let result = self.block_contents(opening.start);
        self.depth -= 1;
        result
    }

    fn block_contents(
        &mut self,
        start: usize,
    ) -> Result<(Block, std::ops::Range<usize>), ExpressionError> {
        let mut statements = Vec::new();
        let mut names = BTreeSet::new();
        while matches!(
            self.current().kind,
            TokenKind::Let | TokenKind::Var | TokenKind::Set
        ) {
            let statement = self.statement()?;
            let declared = match &statement {
                Statement::Let(value) => Some((&value.name, &value.name_span)),
                Statement::Var(value) => Some((&value.name, &value.name_span)),
                Statement::Set(_) => None,
            };
            if declared.is_some_and(|(name, _)| !names.insert(name.clone())) {
                let (name, span) = declared.expect("checked declaration");
                return Err(ExpressionError::new(
                    "EXPRESSION_DUPLICATE_LOCAL",
                    format!("local `{name}` is declared more than once in this block"),
                    span.clone(),
                ));
            }
            statements.push(statement);
        }
        if self.at(&TokenKind::RightBrace) {
            return Err(self.error(
                "EXPRESSION_BLOCK_RESULT",
                "block requires a tail expression",
            ));
        }
        let result = Box::new(self.expression()?);
        if self.at(&TokenKind::Semicolon) {
            return Err(self.error(
                "EXPRESSION_BLOCK_RESULT",
                "block tail expression must not end with `;`",
            ));
        }
        let closing = self.expect(&TokenKind::RightBrace, "}")?;
        Ok((Block { statements, result }, start..closing.end))
    }

    pub(super) fn conditional(
        &mut self,
        start: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.enter_depth(start.clone())?;
        let result = self.conditional_contents(start.start);
        self.depth -= 1;
        result
    }

    fn conditional_contents(&mut self, start: usize) -> Result<Expression, ExpressionError> {
        let previous = self.construct_floor.replace(self.depth);
        let condition = self.expression();
        self.construct_floor = previous;
        let condition = Box::new(condition?);
        let then_open = self.expect(&TokenKind::LeftBrace, "{")?;
        let (then_branch, _) = self.block(then_open)?;
        self.expect(&TokenKind::Else, "else")?;
        let else_open = self.expect(&TokenKind::LeftBrace, "{")?;
        let (else_branch, span) = self.block(else_open)?;
        self.node(
            ExpressionKind::If {
                condition,
                then_branch,
                else_branch,
            },
            start..span.end,
        )
    }
}
