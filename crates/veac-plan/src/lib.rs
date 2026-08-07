//! Deterministic, backend-neutral render plans derived from canonical VEAC projects.

#[cfg(test)]
extern crate self as veac_plan;

mod diagnostic;
mod hash;
mod identity;
mod model;
mod resolver;
mod temporal;
mod validation;

pub use diagnostic::*;
pub use hash::*;
pub use model::*;
pub use resolver::*;
pub use validation::*;
pub use veac_ir as canonical;

pub const RESOLVER_VERSION: &str = "veac-plan-resolver-v6";
pub const EFFECT_REGISTRY_VERSION: &str = "veac-ir-effects-v2";
pub const CAPABILITY_PROFILE: &str = "backend-neutral-v1";
