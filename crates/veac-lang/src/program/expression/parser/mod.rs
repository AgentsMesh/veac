mod binary;
mod closure;
mod collection;
mod control;
mod iteration;
mod r#match;
mod nominal;
mod postfix;
mod primary;
mod range;
mod statement;
mod temporal_attachment;
mod value_type;

use std::mem::discriminant;
use std::ops::Range;

use super::ast::{Expression, ExpressionKind, Statement, UnaryOperator};
use super::lexer::{Token, TokenKind};
use super::{ExpressionError, MAX_EXPRESSION_DEPTH, MAX_EXPRESSION_NODES};

pub(super) fn parse(tokens: Vec<Token>) -> Result<Expression, ExpressionError> {
    let mut parser = Parser::new(tokens);
    let expression = parser.expression()?;
    if !parser.at(&TokenKind::Eof) {
        return Err(parser.error("EXPRESSION_TRAILING_TOKEN", "unexpected trailing token"));
    }
    Ok(expression)
}

pub(super) fn parse_statement(tokens: Vec<Token>) -> Result<Statement, ExpressionError> {
    let mut parser = Parser::new(tokens);
    if !matches!(
        parser.current().kind,
        TokenKind::Let | TokenKind::Var | TokenKind::Set
    ) {
        return Err(parser.error(
            "EXPRESSION_EXPECTED_STATEMENT",
            "expected one `let`, `var`, or `set` statement",
        ));
    }
    let statement = parser.statement()?;
    if !parser.at(&TokenKind::Eof) {
        return Err(parser.error("EXPRESSION_TRAILING_TOKEN", "unexpected trailing token"));
    }
    Ok(statement)
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            nodes: 0,
            depth: 0,
            construct_floor: None,
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    nodes: usize,
    depth: usize,
    construct_floor: Option<usize>,
}

impl Parser {
    fn expression(&mut self) -> Result<Expression, ExpressionError> {
        self.logical_or()
    }

    fn unary(&mut self) -> Result<Expression, ExpressionError> {
        let operator = if let Some(span) = self.take(&TokenKind::Plus) {
            Some((UnaryOperator::Positive, span))
        } else if let Some(span) = self.take(&TokenKind::Minus) {
            Some((UnaryOperator::Negative, span))
        } else {
            self.take(&TokenKind::Bang)
                .map(|span| (UnaryOperator::Not, span))
        };
        let Some((operator, start)) = operator else {
            return self.postfix();
        };
        if operator == UnaryOperator::Negative {
            if let Some(literal) = self.signed_min_literal(start.clone())? {
                return Ok(literal);
            }
        }
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

    fn nominal_allowed(&self) -> bool {
        self.construct_floor.is_none_or(|floor| self.depth > floor)
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

    fn next_at(&self, expected: &TokenKind) -> bool {
        self.tokens
            .get(self.cursor + 1)
            .is_some_and(|token| discriminant(&token.kind) == discriminant(expected))
    }

    fn take(&mut self, expected: &TokenKind) -> Option<Range<usize>> {
        self.at(expected).then(|| self.advance().span)
    }

    fn expect(
        &mut self,
        expected: &TokenKind,
        spelling: &str,
    ) -> Result<Range<usize>, ExpressionError> {
        self.take(expected).ok_or_else(|| {
            self.error(
                "EXPRESSION_EXPECTED_TOKEN",
                format!("expected `{spelling}`"),
            )
        })
    }

    fn error(&self, code: &'static str, message: impl Into<String>) -> ExpressionError {
        ExpressionError::new(code, message, self.current().span.clone())
    }
}
