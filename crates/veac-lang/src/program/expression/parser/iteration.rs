use super::Parser;
use crate::program::expression::ast::{Expression, ExpressionKind};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::ExpressionError;
use crate::vocabulary::{accepts_expression_name, ExpressionNameKind};

impl Parser {
    pub(super) fn iteration(
        &mut self,
        start: std::ops::Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.enter_depth(start.clone())?;
        let result = self.iteration_contents(start.start);
        self.depth -= 1;
        result
    }

    fn iteration_contents(&mut self, start: usize) -> Result<Expression, ExpressionError> {
        let token = self.advance();
        let TokenKind::Symbol(binding) = token.kind else {
            return Err(ExpressionError::new(
                "EXPRESSION_FOR_BINDING",
                "expected a for binding name",
                token.span,
            ));
        };
        if !accepts_expression_name(&binding, ExpressionNameKind::Local) {
            return Err(ExpressionError::new(
                "EXPRESSION_FOR_BINDING",
                format!("`{binding}` is not a valid for binding name"),
                token.span,
            ));
        }
        self.expect(&TokenKind::In, "in")?;
        let previous = self.construct_floor.replace(self.depth);
        let iterable = self.expression();
        self.construct_floor = previous;
        let iterable = Box::new(iterable?);
        let opening = self.expect(&TokenKind::LeftBrace, "{")?;
        let (body, span) = self.block(opening)?;
        self.node(
            ExpressionKind::For {
                binding,
                binding_span: token.span,
                iterable,
                body,
            },
            start..span.end,
        )
    }
}

#[cfg(test)]
#[path = "iteration/tests.rs"]
mod tests;
