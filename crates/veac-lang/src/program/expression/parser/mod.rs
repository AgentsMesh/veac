mod primary;

use std::mem::discriminant;
use std::ops::Range;

use super::ast::{BinaryOperator, Expression, ExpressionKind, UnaryOperator};
use super::lexer::{Token, TokenKind};
use super::{ExpressionError, MAX_EXPRESSION_DEPTH, MAX_EXPRESSION_NODES};

pub(super) fn parse(tokens: Vec<Token>) -> Result<Expression, ExpressionError> {
    let mut parser = Parser {
        tokens,
        cursor: 0,
        nodes: 0,
        depth: 0,
    };
    let expression = parser.expression()?;
    if !parser.at(&TokenKind::Eof) {
        return Err(parser.error("EXPRESSION_TRAILING_TOKEN", "unexpected trailing token"));
    }
    Ok(expression)
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    nodes: usize,
    depth: usize,
}

impl Parser {
    fn expression(&mut self) -> Result<Expression, ExpressionError> {
        self.additive()
    }

    fn additive(&mut self) -> Result<Expression, ExpressionError> {
        let mut expression = self.multiplicative()?;
        loop {
            let operator = if self.take(&TokenKind::Plus).is_some() {
                BinaryOperator::Add
            } else if self.take(&TokenKind::Minus).is_some() {
                BinaryOperator::Subtract
            } else {
                break;
            };
            let right = self.multiplicative()?;
            let span = expression.span.start..right.span.end;
            expression = self.node(
                ExpressionKind::Binary {
                    operator,
                    left: Box::new(expression),
                    right: Box::new(right),
                },
                span,
            )?;
        }
        Ok(expression)
    }

    fn multiplicative(&mut self) -> Result<Expression, ExpressionError> {
        let mut expression = self.unary()?;
        loop {
            let operator = if self.take(&TokenKind::Star).is_some() {
                BinaryOperator::Multiply
            } else if self.take(&TokenKind::Slash).is_some() {
                BinaryOperator::Divide
            } else {
                break;
            };
            let right = self.unary()?;
            let span = expression.span.start..right.span.end;
            expression = self.node(
                ExpressionKind::Binary {
                    operator,
                    left: Box::new(expression),
                    right: Box::new(right),
                },
                span,
            )?;
        }
        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expression, ExpressionError> {
        let operator = if let Some(span) = self.take(&TokenKind::Plus) {
            Some((UnaryOperator::Positive, span))
        } else {
            self.take(&TokenKind::Minus)
                .map(|span| (UnaryOperator::Negative, span))
        };
        let Some((operator, start)) = operator else {
            return self.primary();
        };
        self.enter_depth(start.clone())?;
        let operand = self.unary()?;
        self.depth -= 1;
        let span = start.start..operand.span.end;
        self.node(
            ExpressionKind::Unary {
                operator,
                operand: Box::new(operand),
            },
            span,
        )
    }

    fn node(
        &mut self,
        kind: ExpressionKind,
        span: Range<usize>,
    ) -> Result<Expression, ExpressionError> {
        self.nodes += 1;
        if self.nodes > MAX_EXPRESSION_NODES {
            return Err(ExpressionError::new(
                "EXPRESSION_NODE_LIMIT",
                format!("expression exceeds the {MAX_EXPRESSION_NODES} node limit"),
                span,
            ));
        }
        Ok(Expression { kind, span })
    }

    fn enter_depth(&mut self, span: Range<usize>) -> Result<(), ExpressionError> {
        self.depth += 1;
        if self.depth > MAX_EXPRESSION_DEPTH {
            return Err(ExpressionError::new(
                "EXPRESSION_DEPTH_LIMIT",
                format!("expression exceeds the {MAX_EXPRESSION_DEPTH} nesting limit"),
                span,
            ));
        }
        Ok(())
    }

    fn current(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !self.at(&TokenKind::Eof) {
            self.cursor += 1;
        }
        token
    }

    fn at(&self, expected: &TokenKind) -> bool {
        discriminant(&self.current().kind) == discriminant(expected)
    }

    fn take(&mut self, expected: &TokenKind) -> Option<Range<usize>> {
        self.at(expected).then(|| self.advance().span)
    }

    fn error(&self, code: &'static str, message: impl Into<String>) -> ExpressionError {
        ExpressionError::new(code, message, self.current().span.clone())
    }
}
