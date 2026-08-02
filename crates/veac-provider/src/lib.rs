//! Versioned, deterministic contracts for external analysis and model providers.

mod analysis;
mod artifact;
mod audio;
mod capability;
mod common;
mod envelope;
mod error;
mod proposal;
mod speech;
mod task;
mod validation;
mod vision;

pub use analysis::*;
pub use artifact::*;
pub use audio::*;
pub use capability::*;
pub use common::*;
pub use envelope::*;
pub use error::*;
pub use proposal::*;
pub use speech::*;
pub use task::*;
pub use vision::*;

use schemars::schema_for;

pub const PROVIDER_SCHEMA_ID: &str = "https://veac.dev/schemas/provider-contract";
pub const PROVIDER_SCHEMA_VERSION: u32 = 1;
pub const CAPABILITY_CONTRACT_VERSION: u32 = 1;

pub fn provider_request_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(ProviderRequestEnvelope))
}

pub fn provider_response_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(ProviderResponseEnvelope))
}

pub fn provider_manifest_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(ProviderManifest))
}

pub fn provider_edit_proposal_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(ProviderEditProposal))
}

#[cfg(test)]
#[path = "unit_tests/capability_tests.rs"]
mod capability_tests;
#[cfg(test)]
#[path = "unit_tests/contract_tests.rs"]
mod contract_tests;
#[cfg(test)]
#[path = "unit_tests/domain_tests.rs"]
mod domain_tests;
#[cfg(test)]
#[path = "unit_tests/error_tests.rs"]
mod error_tests;
#[cfg(test)]
#[path = "unit_tests/executable_binding_tests.rs"]
mod executable_binding_tests;
#[cfg(test)]
#[path = "unit_tests/ordering_tests.rs"]
mod ordering_tests;
#[cfg(test)]
#[path = "unit_tests/proposal_tests.rs"]
mod proposal_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
#[path = "unit_tests/validation_branch_tests.rs"]
mod validation_branch_tests;
