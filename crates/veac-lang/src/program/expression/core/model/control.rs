use std::ops::Range;

use super::super::{BlockId, ValueId};
use crate::program::VariantIndex;

mod for_each;
pub use for_each::{
    CoreForEach, CoreForEachEffect, CoreForEachOrder, CoreForEachProvenance, CoreForEachSlot,
    CoreForEachSlotId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreMatchArm {
    pub(crate) variant: VariantIndex,
    pub(crate) target: BlockId,
}

impl CoreMatchArm {
    pub fn variant(&self) -> VariantIndex {
        self.variant
    }

    pub fn target(&self) -> BlockId {
        self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreTerminator {
    Return {
        value: ValueId,
        span: Range<usize>,
    },
    Jump {
        target: BlockId,
        arguments: Vec<ValueId>,
        span: Range<usize>,
    },
    Branch {
        condition: ValueId,
        then_target: BlockId,
        else_target: BlockId,
        span: Range<usize>,
    },
    Match {
        scrutinee: ValueId,
        arms: Vec<CoreMatchArm>,
        span: Range<usize>,
    },
    ForEach(CoreForEach),
}

impl CoreTerminator {
    pub(crate) fn operands(&self) -> std::vec::IntoIter<ValueId> {
        match self {
            Self::Return { value, .. } => vec![*value],
            Self::Jump { arguments, .. } => arguments.clone(),
            Self::Branch { condition, .. } => vec![*condition],
            Self::Match { scrutinee, .. } => vec![*scrutinee],
            Self::ForEach(value) => value.operands().collect(),
        }
        .into_iter()
    }

    pub(crate) fn span(&self) -> &Range<usize> {
        match self {
            Self::Return { span, .. }
            | Self::Jump { span, .. }
            | Self::Branch { span, .. }
            | Self::Match { span, .. } => span,
            Self::ForEach(value) => value.provenance().loop_span(),
        }
    }
}
