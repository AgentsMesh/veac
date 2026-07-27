//! Backend-independent, versioned project model for VEAC timelines.
//!
//! The authoring DSL is intentionally not the persistence format. This crate owns the strict
//! canonical JSON contract shared by agents, editors, migrations, and render-plan builders.

mod audio_contract;
mod canonical;
mod color_contract;
mod edit;
mod effect_contract;
mod id;
mod model;
mod registry;
mod relation_graph;
mod render_budget;
mod strict_json;
mod text_contract;
mod time;
mod validation;
mod visual_contract;

pub use audio_contract::*;
pub use canonical::*;
pub use color_contract::*;
pub use edit::*;
pub use effect_contract::*;
pub use id::*;
pub use model::*;
pub use registry::*;
pub use relation_graph::*;
pub use render_budget::*;
pub use text_contract::*;
pub use time::*;
pub use validation::*;
pub use visual_contract::*;

pub const SCHEMA_ID: &str = "https://veac.dev/schemas/project";
pub const CURRENT_SCHEMA_VERSION: u32 = 3;
pub const MIN_READER_VERSION: u32 = 3;
/// Largest integer represented exactly by the IEEE-754 number domain required by RFC 8785/I-JSON.
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub const MAX_SEQUENCE_NESTING_DEPTH: usize = 64;
pub const MAX_MATTE_NESTING_DEPTH: usize = 64;

/// Reject duplicate object names and trailing JSON before typed deserialization.
pub fn reject_duplicate_json_keys(input: &str) -> Result<(), serde_json::Error> {
    strict_json::reject_duplicate_keys(input)
}

#[cfg(test)]
mod test_support;

#[cfg(test)]
#[path = "unit_tests/annotation_domain_tests.rs"]
mod annotation_domain_tests;
#[cfg(test)]
#[path = "unit_tests/annotation_payload_tests.rs"]
mod annotation_payload_tests;
#[cfg(test)]
#[path = "unit_tests/annotation_tests.rs"]
mod annotation_tests;
#[cfg(test)]
#[path = "unit_tests/canonical_tests.rs"]
mod canonical_tests;
#[cfg(test)]
#[path = "unit_tests/edit_contract_tests.rs"]
mod edit_contract_tests;
#[cfg(test)]
#[path = "unit_tests/edit_tests.rs"]
mod edit_tests;
#[cfg(test)]
#[path = "unit_tests/id_time_tests.rs"]
mod id_time_tests;
#[cfg(test)]
#[path = "unit_tests/model_tests.rs"]
mod model_tests;
#[cfg(test)]
#[path = "unit_tests/output_model_tests.rs"]
mod output_model_tests;
#[cfg(test)]
#[path = "unit_tests/professional_aux_output_tests.rs"]
mod professional_aux_output_tests;
#[cfg(test)]
#[path = "unit_tests/professional_output_tests.rs"]
mod professional_output_tests;
#[cfg(test)]
#[path = "unit_tests/registry_edge_tests.rs"]
mod registry_edge_tests;
#[cfg(test)]
#[path = "unit_tests/registry_tests.rs"]
mod registry_tests;
#[cfg(test)]
#[path = "unit_tests/relation_tests.rs"]
mod relation_tests;
#[cfg(test)]
#[path = "unit_tests/strict_json_tests.rs"]
mod strict_json_tests;
#[cfg(test)]
#[path = "unit_tests/validation_tests.rs"]
mod validation_tests;
