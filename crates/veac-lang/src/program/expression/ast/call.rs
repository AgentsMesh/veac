use std::ops::Range;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::program::expression) struct CallArgumentLabel {
    pub name: String,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::program::expression) struct CallArgument {
    pub label: Option<CallArgumentLabel>,
    pub value: Expression,
}
