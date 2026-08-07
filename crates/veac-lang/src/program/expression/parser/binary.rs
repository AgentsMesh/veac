use super::Parser;
use crate::program::expression::ast::{BinaryOperator, Expression, ExpressionKind};
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::ExpressionError;

impl Parser {
    pub(super) fn logical_or(&mut self) -> Result<Expression, ExpressionError> {
        self.binary_chain(
            Self::logical_and,
            &[(TokenKind::OrOr, BinaryOperator::LogicalOr)],
        )
    }

    fn logical_and(&mut self) -> Result<Expression, ExpressionError> {
        self.binary_chain(
            Self::equality,
            &[(TokenKind::AndAnd, BinaryOperator::LogicalAnd)],
        )
    }

    fn equality(&mut self) -> Result<Expression, ExpressionError> {
        self.non_associative(
            Self::ordering,
            &[
                (TokenKind::EqualEqual, BinaryOperator::Equal),
                (TokenKind::BangEqual, BinaryOperator::NotEqual),
            ],
            "equality",
        )
    }

    fn ordering(&mut self) -> Result<Expression, ExpressionError> {
        self.non_associative(
            Self::range,
            &[
                (TokenKind::Less, BinaryOperator::Less),
                (TokenKind::LessEqual, BinaryOperator::LessEqual),
                (TokenKind::Greater, BinaryOperator::Greater),
                (TokenKind::GreaterEqual, BinaryOperator::GreaterEqual),
            ],
            "ordering",
        )
    }

    pub(super) fn additive(&mut self) -> Result<Expression, ExpressionError> {
        self.binary_chain(
            Self::multiplicative,
            &[
                (TokenKind::Plus, BinaryOperator::Add),
                (TokenKind::Minus, BinaryOperator::Subtract),
            ],
        )
    }

    fn multiplicative(&mut self) -> Result<Expression, ExpressionError> {
        self.binary_chain(
            Self::unary,
            &[
                (TokenKind::Star, BinaryOperator::Multiply),
                (TokenKind::Slash, BinaryOperator::Divide),
            ],
        )
    }

    fn non_associative(
        &mut self,
        operand: fn(&mut Self) -> Result<Expression, ExpressionError>,
        operators: &[(TokenKind, BinaryOperator)],
        family: &str,
    ) -> Result<Expression, ExpressionError> {
        let left = operand(self)?;
        let Some(operator) = self.take_operator(operators) else {
            return Ok(left);
        };
        let right = operand(self)?;
        let expression = self.combine(left, operator, right)?;
        if self.matches_any(operators) {
            return Err(self.error(
                "EXPRESSION_COMPARISON_CHAIN",
                format!("{family} operators cannot be chained"),
            ));
        }
        Ok(expression)
    }

    fn binary_chain(
        &mut self,
        operand: fn(&mut Self) -> Result<Expression, ExpressionError>,
        operators: &[(TokenKind, BinaryOperator)],
    ) -> Result<Expression, ExpressionError> {
        let mut expression = operand(self)?;
        while let Some(operator) = self.take_operator(operators) {
            let right = operand(self)?;
            expression = self.combine(expression, operator, right)?;
        }
        Ok(expression)
    }

    fn combine(
        &mut self,
        left: Expression,
        operator: BinaryOperator,
        right: Expression,
    ) -> Result<Expression, ExpressionError> {
        let span = left.span.start..right.span.end;
        self.node(
            ExpressionKind::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        )
    }

    fn take_operator(
        &mut self,
        operators: &[(TokenKind, BinaryOperator)],
    ) -> Option<BinaryOperator> {
        operators
            .iter()
            .find_map(|(token, operator)| self.take(token).map(|_| *operator))
    }

    fn matches_any(&self, operators: &[(TokenKind, BinaryOperator)]) -> bool {
        operators.iter().any(|(token, _)| self.at(token))
    }
}
