use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_artifact::ContentDigest;
use veac_ir::EditBatch;

use crate::{ProviderOutput, Validate};

mod build;
mod context;
mod evidence;
mod evidence_validate;
mod time;
mod validate;

pub use build::propose_edit;
pub use context::*;
pub use evidence::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderEditProposal {
    pub request_hash: ContentDigest,
    pub response_hash: ContentDigest,
    pub source_request: Box<crate::ProviderRequestEnvelope>,
    pub application_context: Box<ApplicationContext>,
    #[schemars(range(max = 9007199254740991u64))]
    pub project_revision: u64,
    pub project_timebase: u32,
    pub source_output: ProviderOutput,
    pub evidence: Vec<ProposalEvidence>,
    pub batch: EditBatch,
}

pub fn canonical_edit_proposal_bytes(
    value: &ProviderEditProposal,
) -> crate::ProviderResult<Vec<u8>> {
    value.validate()?;
    serde_json_canonicalizer::to_vec(value).map_err(Into::into)
}
