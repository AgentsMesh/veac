//! Versioned, backend-neutral subtitle interchange for canonical VEAC timelines.

mod canonical;
mod error;
mod export;
mod format;
mod import;
mod ir;
mod model;
mod time;
mod validation;

pub use canonical::*;
pub use error::*;
pub use export::*;
pub use format::*;
pub use import::*;
pub use ir::*;
pub use model::*;
pub use validation::*;

pub const SCHEMA_ID: &str = "https://veac.dev/schemas/caption-document";
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

#[cfg(test)]
#[path = "unit_tests/canonical_tests.rs"]
mod canonical_tests;
#[cfg(test)]
#[path = "unit_tests/export_native_tests.rs"]
mod export_native_tests;
#[cfg(test)]
#[path = "unit_tests/export_tests.rs"]
mod export_tests;
#[cfg(test)]
#[path = "unit_tests/import_tests.rs"]
mod import_tests;
#[cfg(test)]
#[path = "unit_tests/ir_tests.rs"]
mod ir_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
#[path = "unit_tests/validation_tests.rs"]
mod validation_tests;
