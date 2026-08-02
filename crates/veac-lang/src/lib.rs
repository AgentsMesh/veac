//! Agent-oriented VEAC authoring language lowered directly to canonical IR.

pub mod authoring;

pub use authoring::{format_document, lower_document, parse, Diagnostics, Document};
