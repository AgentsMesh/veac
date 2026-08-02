//! Deterministic, backend-neutral render plans derived from canonical VEAC projects.

#[cfg(test)]
extern crate self as veac_plan;

mod diagnostic;
mod hash;
mod model;
mod resolver;

pub use diagnostic::*;
pub use hash::*;
pub use model::*;
pub use resolver::*;
pub use veac_ir as canonical;

pub const RESOLVER_VERSION: &str = "veac-plan-resolver-v5";
