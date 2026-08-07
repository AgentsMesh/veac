use std::ops::Range;

use super::Value;
use crate::program::TypeSyntax;

mod nominal;
mod path;

#[cfg(test)]
mod tests;

pub(super) use nominal::{MatchArm, MatchPattern, NominalField, PatternField};
pub(super) use path::{join_path, static_path, PathSegment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UnaryOperator {
    Positive,
    Negative,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LetBinding {
    pub name: String,
    pub name_span: Range<usize>,
    pub annotation: Option<TypeAnnotation>,
    pub value: Expression,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MutableBinding {
    pub name: String,
    pub name_span: Range<usize>,
    pub annotation: Option<TypeAnnotation>,
    pub value: Expression,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MutableAssignment {
    pub name: String,
    pub name_span: Range<usize>,
    pub value: Expression,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Statement {
    Let(LetBinding),
    Var(MutableBinding),
    Set(MutableAssignment),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TypeAnnotation {
    pub syntax: TypeSyntax,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ClosureParameter {
    pub name: String,
    pub name_span: Range<usize>,
    pub annotation: TypeAnnotation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MapEntry {
    pub key: Expression,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Block {
    pub statements: Vec<Statement>,
    pub result: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TemporalAttachment {
    pub property: crate::program::model::TemporalProperty,
    pub target: super::TemporalTargetKind,
    pub arguments: Vec<Expression>,
    pub body: Block,
    pub body_span: Range<usize>,
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
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        step: Option<Box<Expression>>,
    },
    Closure {
        parameters: Vec<ClosureParameter>,
        return_type: Box<TypeAnnotation>,
        effect: super::FunctionEffect,
        body: Block,
    },
    For {
        binding: String,
        binding_span: Range<usize>,
        iterable: Box<Expression>,
        body: Block,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    FieldProject {
        receiver: Box<Expression>,
        field: String,
        field_span: Range<usize>,
    },
    NominalConstruct {
        path: Vec<PathSegment>,
        fields: Vec<NominalField>,
    },
    Match {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    TemporalAttach(Box<TemporalAttachment>),
    List(Vec<Expression>),
    Map(Vec<MapEntry>),
    Tuple(Vec<Expression>),
    Block(Block),
    If {
        condition: Box<Expression>,
        then_branch: Block,
        else_branch: Block,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Expression {
    pub kind: ExpressionKind,
    pub span: Range<usize>,
}
