use super::Parser;
use crate::program::expression::ast::{Expression, ExpressionKind};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::ExpressionError;

impl Parser {
    pub(super) fn range(&mut self) -> Result<Expression, ExpressionError> {
        let start = self.additive()?;
        if self.take(&TokenKind::DotDot).is_none() {
            return Ok(start);
        }
        let end = self.additive()?;
        let step = self
            .take(&TokenKind::By)
            .map(|_| self.additive().map(Box::new))
            .transpose()?;
        if self.at(&TokenKind::DotDot) {
            return Err(self.error(
                "EXPRESSION_RANGE_CHAIN",
                "range operators cannot be chained",
            ));
        }
        let span = start.span.start..step.as_deref().unwrap_or(&end).span.end;
        self.node(
            ExpressionKind::Range {
                start: Box::new(start),
                end: Box::new(end),
                step,
            },
            span,
        )
    }
}

#[cfg(test)]
#[path = "range/tests.rs"]
mod tests;
