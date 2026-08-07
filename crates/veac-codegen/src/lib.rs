pub mod emitter;
mod identity;

pub use identity::*;

/// Bump when emitted rendering semantics change enough to invalidate cached media.
pub const RENDER_IMPLEMENTATION_CONTRACT_VERSION: u32 = 1;

#[cfg(test)]
#[path = "../build_identity.rs"]
mod build_identity_contract;
#[cfg(test)]
mod build_identity_tests;

#[cfg(test)]
extern crate self as veac_codegen;

#[cfg(test)]
mod unit_tests;
