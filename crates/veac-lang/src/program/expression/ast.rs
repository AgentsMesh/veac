use std::ops::Range;

use super::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UnaryOperator {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ExpressionKind {
    Literal(Value),
    Symbol(String),
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Expression {
    pub kind: ExpressionKind,
    pub span: Range<usize>,
}
