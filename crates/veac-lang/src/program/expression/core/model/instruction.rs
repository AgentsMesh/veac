use std::ops::Range;

use super::super::{
    ClosureDefinitionId, CoreTypeId, CoreValueMetadata, FunctionId, InputId, LocalSlotId, ValueId,
};
use crate::program::expression::{BuiltinFunction, CollectionOperation, Value};
use crate::program::{FieldIndex, TypeId, VariantIndex};

mod operands;
mod temporal;
pub use temporal::{CoreTemporalComposeOperation, CoreTemporalProjectOperation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreUnaryOperator {
    Positive,
    Negative,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualityOperator {
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreCallTarget {
    Builtin(BuiltinFunction),
    User(FunctionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreInstructionKind {
    Literal(Value),
    Input(InputId),
    Parameter(usize),
    Capture(usize),
    LocalInit {
        slot: LocalSlotId,
        value: ValueId,
    },
    LocalSet {
        slot: LocalSlotId,
        value: ValueId,
    },
    LocalGet {
        slot: LocalSlotId,
    },
    Closure {
        definition: ClosureDefinitionId,
        captures: Vec<ValueId>,
    },
    Invoke {
        callee: ValueId,
        arguments: Vec<ValueId>,
    },
    Unary {
        operator: CoreUnaryOperator,
        operand: ValueId,
    },
    Arithmetic {
        operator: ArithmeticOperator,
        left: ValueId,
        right: ValueId,
    },
    Compare {
        operator: ComparisonOperator,
        left: ValueId,
        right: ValueId,
    },
    Equal {
        operator: EqualityOperator,
        left: ValueId,
        right: ValueId,
    },
    Call {
        target: CoreCallTarget,
        arguments: Vec<ValueId>,
    },
    Collection {
        operation: CollectionOperation,
        iterable: ValueId,
        initial: Option<ValueId>,
        callable: ValueId,
    },
    StructConstruct {
        type_id: TypeId,
        fields: Vec<ValueId>,
    },
    StructProject {
        structure: ValueId,
        field: FieldIndex,
    },
    EnumConstruct {
        type_id: TypeId,
        variant: VariantIndex,
        fields: Vec<ValueId>,
    },
    DomainConstruct {
        opcode: u16,
        operands: Vec<ValueId>,
    },
    GraphEmit {
        opcode: u16,
        operands: Vec<ValueId>,
    },
    TemporalAttach {
        kind: u8,
        owner: ValueId,
        selectors: Vec<ValueId>,
        animation: ValueId,
    },
    TemporalCompose {
        operation: CoreTemporalComposeOperation,
        operands: Vec<ValueId>,
    },
    TemporalProject {
        operation: CoreTemporalProjectOperation,
        value: ValueId,
    },
    List {
        elements: Vec<ValueId>,
    },
    Tuple {
        elements: Vec<ValueId>,
    },
    Range {
        start: ValueId,
        end: ValueId,
        step: Option<ValueId>,
    },
    MapBegin {
        entries: u32,
    },
    MapKey {
        builder: ValueId,
        key: ValueId,
        ordinal: u32,
    },
    MapValue {
        pending: ValueId,
        value: ValueId,
    },
    MapFinish {
        builder: ValueId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreInstruction {
    pub(crate) id: ValueId,
    pub(crate) kind: CoreInstructionKind,
    pub(crate) type_id: CoreTypeId,
    pub(crate) metadata: CoreValueMetadata,
    pub(crate) span: Range<usize>,
}

impl CoreInstruction {
    pub fn id(&self) -> ValueId {
        self.id
    }

    pub fn kind(&self) -> &CoreInstructionKind {
        &self.kind
    }

    pub fn type_id(&self) -> CoreTypeId {
        self.type_id
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn metadata(&self) -> &CoreValueMetadata {
        &self.metadata
    }
}
