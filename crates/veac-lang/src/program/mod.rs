//! Deterministic compile-time modules, expressions, presets, and components.

mod compile;
mod dependency_budget;
mod diagnostic;
mod expand;
pub mod expression;
mod index;
mod lexer;
mod limits;
mod loader;
mod model;
mod parser;
mod provenance;
mod resolve;
mod source_transaction;
mod token;

pub use compile::{
    check_source, compile_path, compile_path_with_root, compile_source, compile_with_loader,
    CompiledProgram,
};
pub use diagnostic::{Diagnostic, Diagnostics};
pub(crate) use index::validate_expression_fragment;
pub use index::*;
pub use loader::{FileSystemLoader, LoadedSource, SourceLoader};
pub use provenance::{Origin, ProvenanceMap};
pub use source_transaction::{
    apply_source_edit_path, apply_source_edit_path_with_root, SourceEditPreview,
    SourceTransactionError,
};

#[cfg(test)]
mod tests;
