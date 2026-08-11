//! Versioned project manifests and deterministic project graph resolution.

mod authored;
mod canonical;
mod error;
pub mod model;
mod resolve;
mod validate;

pub use authored::*;
pub use canonical::{canonical_manifest_bytes, canonical_manifest_json, manifest_digest};
pub use error::{IssueCode, ProjectIssue, ProjectIssues};
pub use model::*;
pub use resolve::resolve_manifest;
pub use validate::validate_manifest;

pub const PROJECT_SCHEMA: &str = "veac.project";
pub const PROJECT_MANIFEST_VERSION: u32 = 1;
pub const RESOLVED_GRAPH_VERSION: u32 = 1;

pub fn project_manifest_json_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(ProjectManifestV1))
        .expect("ProjectManifestV1 schema is serializable")
}

pub fn resolved_project_graph_json_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(ResolvedTargetGraph))
        .expect("ResolvedTargetGraph schema is serializable")
}

pub fn authored_project_source_index_json_schema() -> serde_json::Value {
    veac_lang::program::source_index_json_schema()
        .expect("SourceIndexInventory schema is serializable")
}
