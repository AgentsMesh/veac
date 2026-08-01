pub mod emitter;

/// Bump when emitted rendering semantics change enough to invalidate cached media.
pub const RENDER_IMPLEMENTATION_CONTRACT_VERSION: u32 = 1;

#[cfg(test)]
extern crate self as veac_codegen;

#[cfg(test)]
mod unit_tests;
