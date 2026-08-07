use std::ops::Range;
use std::sync::Arc;

use super::ast::{BinaryOperator, UnaryOperator};
use super::core::FunctionId;
use super::{BuiltinFunction, CollectionOperation, TemporalAttachmentKind, Value, ValueType};
use crate::program::DomainOperationRegistry;
use crate::program::TypeRegistry;

mod nominal;

pub(super) use nominal::{
    TypedEnumConstruct, TypedMatchArm, TypedMethodCall, TypedPatternBinding, TypedStructConstruct,
    TypedStructProject,
};

#[derive(Debug, Clone)]
pub(super) struct TypedExpression {
    pub root: TypedNode,
    pub types: Arc<TypeRegistry>,
    pub domain: Arc<DomainOperationRegistry>,
}

impl TypedExpression {
    pub(super) fn result_type(&self) -> &ValueType {
        &self.root.value_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct LocalId(u32);

impl LocalId {
    pub(super) fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MutableLocalId(u32);

impl MutableLocalId {
    pub(super) fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub(super) struct TypedNode {
    pub kind: TypedNodeKind,
    pub value_type: ValueType,
    pub span: Range<usize>,
}

#[derive(Debug, Clone)]
pub(super) struct TypedBinding {
    pub id: LocalId,
    pub value: TypedNode,
}

#[derive(Debug, Clone)]
pub(super) struct TypedMutableBinding {
    pub id: MutableLocalId,
    pub value: TypedNode,
}

#[derive(Debug, Clone)]
pub(super) struct TypedMutableAssignment {
    pub id: MutableLocalId,
    pub value: TypedNode,
}

#[derive(Debug, Clone)]
pub(super) enum TypedStatement {
    Let(TypedBinding),
    Var(TypedMutableBinding),
    Set(TypedMutableAssignment),
}

#[derive(Debug, Clone)]
pub(super) struct TypedBlock {
    pub statements: Vec<TypedStatement>,
    pub result: Box<TypedNode>,
}

#[derive(Debug, Clone)]
pub(super) struct TypedMapEntry {
    pub key: TypedNode,
    pub value: TypedNode,
}

#[derive(Debug, Clone)]
pub(super) struct TypedClosureParameter {
    pub value_type: ValueType,
    pub stage: super::core::Stage,
}

#[derive(Debug, Clone)]
pub(super) struct TypedCapture {
    pub source: TypedNode,
}

#[derive(Debug, Clone)]
pub(super) struct TypedDomainCall {
    pub opcode: u16,
    pub operands: Vec<TypedNode>,
}

#[derive(Debug, Clone)]
pub(super) struct TypedTemporalAttach {
    pub kind: TemporalAttachmentKind,
    pub owner: Box<TypedNode>,
    pub selectors: Vec<TypedNode>,
    pub animation: Box<TypedNode>,
}

#[derive(Debug, Clone)]
pub(super) struct TypedIteration {
    pub binding_span: Range<usize>,
}

#[derive(Debug, Clone)]
pub(super) enum TypedNodeKind {
    Literal(Value),
    External(String),
    Parameter(usize),
    Local(LocalId),
    MutableLocal(MutableLocalId),
    Capture(usize),
    Unary {
        operator: UnaryOperator,
        operand: Box<TypedNode>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<TypedNode>,
        right: Box<TypedNode>,
    },
    Range {
        start: Box<TypedNode>,
        end: Box<TypedNode>,
        step: Option<Box<TypedNode>>,
    },
    Closure {
        parameters: Vec<TypedClosureParameter>,
        captures: Vec<TypedCapture>,
        body: TypedBlock,
        non_escaping: bool,
    },
    Call {
        target: CallTarget,
        arguments: Vec<TypedNode>,
    },
    Invoke {
        callee: Box<TypedNode>,
        arguments: Vec<TypedNode>,
    },
    MethodCall(TypedMethodCall),
    DomainCall(TypedDomainCall),
    TemporalAttach(TypedTemporalAttach),
    StructConstruct(TypedStructConstruct),
    EnumConstruct(TypedEnumConstruct),
    StructProject(TypedStructProject),
    Match {
        scrutinee: Box<TypedNode>,
        arms: Vec<TypedMatchArm>,
    },
    Collection {
        operation: CollectionOperation,
        arguments: Vec<TypedNode>,
    },
    ForEach {
        iterable: Box<TypedNode>,
        body: Box<TypedNode>,
        iteration: TypedIteration,
    },
    List(Vec<TypedNode>),
    Map(Vec<TypedMapEntry>),
    Tuple(Vec<TypedNode>),
    Block(TypedBlock),
    If {
        condition: Box<TypedNode>,
        then_branch: TypedBlock,
        else_branch: TypedBlock,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) enum CallTarget {
    Builtin(BuiltinFunction),
    User(FunctionId),
}
