//! Agent-oriented VEAC authoring language lowered directly to canonical IR.

pub mod authoring;
mod name;
pub mod program;
pub mod source_edit;
mod string_codec;

pub use authoring::{format_document, lower_document, parse, Diagnostics, Document};
