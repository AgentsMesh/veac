mod selection;
mod sinks;

use veac_ir::{ItemId, SequenceId, TemporalBindingId, TemporalType};

pub(crate) use selection::select;
pub(crate) use sinks::collect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemporalSink {
    pub binding_id: TemporalBindingId,
    pub expected: TemporalType,
    pub pointer: String,
    pub scope: TemporalSinkScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemporalSinkScope {
    pub sequence_id: SequenceId,
    pub item_id: Option<ItemId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemporalIssue {
    pub code: &'static str,
    pub pointer: String,
    pub message: &'static str,
}

impl TemporalIssue {
    pub(crate) fn new(code: &'static str, pointer: String, message: &'static str) -> Self {
        Self {
            code,
            pointer,
            message,
        }
    }
}
