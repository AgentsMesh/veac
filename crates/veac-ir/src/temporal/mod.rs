mod binding;
mod digest;
mod error;
mod evaluation;
mod id;
mod input;
mod node;
mod operation;
mod program;
mod provenance;
mod validation;
mod value;

pub use binding::*;
pub use digest::*;
pub use error::*;
pub use evaluation::*;
pub use id::*;
pub use input::*;
pub use node::*;
pub use operation::*;
pub use program::*;
pub use provenance::*;
pub use validation::*;
pub use value::*;

pub const TEMPORAL_OPSET_VERSION: u16 = 1;
pub const MAX_TEMPORAL_PROGRAMS: usize = 4_096;
pub const MAX_TEMPORAL_BINDINGS: usize = 65_536;
pub const MAX_TEMPORAL_INPUTS: usize = 256;
pub const MAX_TEMPORAL_NODES: usize = 262_144;
pub const MAX_TEMPORAL_NODES_PER_PROGRAM: usize = 65_536;
pub const MAX_TEMPORAL_CURVE_KEYS: usize = 4_096;
pub const MAX_TEMPORAL_PROVENANCE: usize = 262_144;
pub const MAX_TEMPORAL_CALL_DEPTH: usize = 64;
pub const MAX_TEMPORAL_LOGICAL_KEYS: usize = 64;
pub const MAX_TEMPORAL_TEXT_BYTES: usize = 1_048_576;
pub const MAX_TEMPORAL_VALUE_BYTES: usize = 64 * 1024 * 1024;

#[cfg(test)]
mod tests;
