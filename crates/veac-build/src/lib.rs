//! Deterministic, cache-aware execution for VEAC artifact graphs.

mod action;
mod cache;
mod cancellation;
mod error;
mod executor;
mod graph;
mod id;
mod output;
mod project;
mod receipt;
mod resource;
mod scheduler;
mod stable;

pub use action::*;
pub use cache::*;
pub use cancellation::*;
pub use error::*;
pub use executor::*;
pub use graph::*;
pub use id::*;
pub use output::*;
pub use project::*;
pub use receipt::*;
pub use resource::*;
pub use scheduler::*;
pub use veac_artifact::ContentDigest;

pub const BUILD_GRAPH_CONTRACT_VERSION: u32 = 2;
pub const MAX_ACTION_BYTES: usize = 1024 * 1024;
pub const MAX_GRAPH_EDGES: usize = 16_384;
pub const MAX_GRAPH_NODES: usize = 4_096;
pub const MAX_NODE_OUTPUTS: usize = 128;

#[cfg(test)]
#[path = "unit_tests/cache_poison_tests.rs"]
mod cache_poison_tests;
