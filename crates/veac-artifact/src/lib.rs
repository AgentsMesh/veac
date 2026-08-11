//! Reproducible build manifests and content-addressed artifacts for VEAC.

mod binding;
mod cache;
mod delivery_package;
mod descriptor;
mod digest;
mod error;
mod execution;
mod json;
mod limits;
mod manifest;
mod materialize;
mod package;
mod relink;
mod render_segment;
mod schema;
mod source;
mod staged;
mod workflow;

pub use binding::*;
pub use cache::*;
pub use delivery_package::*;
pub use descriptor::*;
pub use digest::*;
pub use error::*;
pub use execution::*;
pub use json::{canonical_artifact_json_bounded, validate_artifact_json};
pub use limits::*;
pub use manifest::*;
pub use materialize::*;
pub use package::*;
pub use relink::*;
pub use render_segment::*;
pub use schema::*;
pub use source::*;
pub use staged::*;
pub use workflow::*;

pub const ARTIFACT_CONTRACT_VERSION: u32 = 3;
pub const ARTIFACT_SCHEMA_ID: &str = "https://veac.dev/schemas/artifact";
pub const BUILD_MANIFEST_SCHEMA_ID: &str = "https://veac.dev/schemas/build-manifest";
pub const PACKAGE_MANIFEST_SCHEMA_ID: &str = "https://veac.dev/schemas/package-manifest";

#[cfg(test)]
#[path = "unit_tests/artifact_parameter_analysis_tests.rs"]
mod artifact_parameter_analysis_tests;
#[cfg(test)]
#[path = "unit_tests/artifact_parameter_dispatch_tests.rs"]
mod artifact_parameter_dispatch_tests;
#[cfg(test)]
#[path = "unit_tests/artifact_parameter_evidence_tests.rs"]
mod artifact_parameter_evidence_tests;
#[cfg(test)]
#[path = "unit_tests/artifact_parameter_provider_tests.rs"]
mod artifact_parameter_provider_tests;
#[cfg(test)]
#[path = "unit_tests/artifact_parameter_render_tests.rs"]
mod artifact_parameter_render_tests;
#[cfg(test)]
#[path = "unit_tests/binding_contract_edge_tests.rs"]
mod binding_contract_edge_tests;
#[cfg(test)]
#[path = "unit_tests/binding_tests.rs"]
mod binding_tests;
#[cfg(test)]
#[path = "unit_tests/cache_file_guard_tests.rs"]
mod cache_file_guard_tests;
#[cfg(test)]
#[path = "unit_tests/cache_guard_tests.rs"]
mod cache_guard_tests;
#[cfg(test)]
#[path = "unit_tests/cache_metadata_tests.rs"]
mod cache_metadata_tests;
#[cfg(test)]
#[path = "unit_tests/cache_safety_tests.rs"]
mod cache_safety_tests;
#[cfg(test)]
#[path = "unit_tests/cache_tests.rs"]
mod cache_tests;
#[cfg(test)]
#[path = "unit_tests/catalog_path_tests.rs"]
mod catalog_path_tests;
#[cfg(test)]
#[path = "unit_tests/catalog_tests.rs"]
mod catalog_tests;
#[cfg(test)]
#[path = "unit_tests/contract_tests.rs"]
mod contract_tests;
#[cfg(test)]
#[path = "unit_tests/delivery_package_tests.rs"]
mod delivery_package_tests;
#[cfg(test)]
#[path = "unit_tests/error_mapping_tests.rs"]
mod error_mapping_tests;
#[cfg(test)]
#[path = "unit_tests/execution_binding_tests.rs"]
mod execution_binding_tests;
#[cfg(test)]
#[path = "unit_tests/execution_bindings_edge_tests.rs"]
mod execution_bindings_edge_tests;
#[cfg(test)]
#[path = "unit_tests/manifest_tests.rs"]
mod manifest_tests;
#[cfg(test)]
#[path = "unit_tests/materialize_edge_tests.rs"]
mod materialize_edge_tests;
#[cfg(test)]
#[path = "unit_tests/materialize_guard_tests.rs"]
mod materialize_guard_tests;
#[cfg(test)]
#[path = "unit_tests/materialize_tests.rs"]
mod materialize_tests;
#[cfg(test)]
#[path = "unit_tests/package_tests.rs"]
mod package_tests;
#[cfg(test)]
#[path = "unit_tests/proxy_selection_tests.rs"]
mod proxy_selection_tests;
#[cfg(test)]
#[path = "unit_tests/put_file_tests.rs"]
mod put_file_tests;
#[cfg(test)]
#[path = "unit_tests/render_segment_edge_tests.rs"]
mod render_segment_edge_tests;
#[cfg(test)]
#[path = "unit_tests/render_segment_tests.rs"]
mod render_segment_tests;
#[cfg(test)]
#[path = "unit_tests/resource_read_tests.rs"]
mod resource_read_tests;
#[cfg(test)]
#[path = "unit_tests/schema_tests.rs"]
mod schema_tests;
#[cfg(test)]
#[path = "unit_tests/source_clock_tests.rs"]
mod source_clock_tests;
#[cfg(test)]
#[path = "unit_tests/source_copy_stage_tests.rs"]
mod source_copy_stage_tests;
#[cfg(test)]
#[path = "unit_tests/source_limits_tests.rs"]
mod source_limits_tests;
#[cfg(test)]
#[path = "unit_tests/source_snapshot_tests.rs"]
mod source_snapshot_tests;
#[cfg(test)]
#[path = "unit_tests/source_success_tests.rs"]
mod source_success_tests;
#[cfg(test)]
#[path = "unit_tests/source_verify_tests.rs"]
mod source_verify_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
#[path = "unit_tests/verified_artifact_tests.rs"]
mod verified_artifact_tests;
#[cfg(test)]
#[path = "unit_tests/workflow_tests.rs"]
mod workflow_tests;
