//! Typed, deterministic template-slot fill proposals for canonical VEAC projects.

mod bindings;
mod error;
mod geometry;
mod inventory;
mod media;
mod proposal;
mod request;
mod timing;

pub use error::*;
pub use proposal::propose_template_fill;
pub use request::*;

#[cfg(test)]
extern crate self as veac_template;
#[cfg(test)]
mod unit_tests;
