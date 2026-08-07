use crate::program::expression::hir::{LocalId, MutableLocalId, TypedCapture, TypedNodeKind};
use crate::program::expression::ValueType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::program::expression::compile::lower) enum BindingKey {
    Local(LocalId),
    Mutable(MutableLocalId),
    Parameter { owner: usize, index: usize },
}

#[derive(Debug, Clone, Copy)]
pub(in crate::program::expression::compile::lower) enum BindingKind {
    Local(LocalId),
    Mutable(MutableLocalId),
    Parameter(usize),
}

impl BindingKind {
    pub(super) fn node_kind(self) -> TypedNodeKind {
        match self {
            Self::Local(id) => TypedNodeKind::Local(id),
            Self::Mutable(id) => TypedNodeKind::MutableLocal(id),
            Self::Parameter(index) => TypedNodeKind::Parameter(index),
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::program::expression::compile::lower) struct ScopeBinding {
    pub key: BindingKey,
    pub kind: BindingKind,
    pub owner: usize,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Default)]
pub(in crate::program::expression::compile::lower) struct ClosureContext {
    pub keys: Vec<BindingKey>,
    pub captures: Vec<TypedCapture>,
    pub allows_function_captures: bool,
}

impl ClosureContext {
    pub(in crate::program::expression::compile::lower) fn iteration() -> Self {
        Self {
            allows_function_captures: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::program::expression::compile::lower) struct Initializer {
    pub name: String,
    pub owner: usize,
}

pub(in crate::program::expression::compile::lower) struct Checkpoint {
    pub next_local: u32,
    pub next_mutable: u32,
    pub capture_lengths: Vec<usize>,
}

impl ScopeBinding {
    pub(in crate::program::expression::compile::lower) fn local(
        id: LocalId,
        value_type: ValueType,
        owner: usize,
    ) -> Self {
        Self {
            key: BindingKey::Local(id),
            kind: BindingKind::Local(id),
            owner,
            value_type,
        }
    }

    pub(in crate::program::expression::compile::lower) fn parameter(
        index: usize,
        value_type: ValueType,
        owner: usize,
    ) -> Self {
        Self {
            key: BindingKey::Parameter { owner, index },
            kind: BindingKind::Parameter(index),
            owner,
            value_type,
        }
    }

    pub(in crate::program::expression::compile::lower) fn mutable(
        id: MutableLocalId,
        value_type: ValueType,
        owner: usize,
    ) -> Self {
        Self {
            key: BindingKey::Mutable(id),
            kind: BindingKind::Mutable(id),
            owner,
            value_type,
        }
    }
}
