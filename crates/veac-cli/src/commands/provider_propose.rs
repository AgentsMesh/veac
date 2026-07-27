use veac_provider::{
    canonical_edit_proposal_bytes, propose_edit, CaptionProposalContext, ProviderRequestEnvelope,
    ProviderResponseEnvelope,
};

use crate::arguments::ProviderProposeArgs;
use crate::CliResult;

pub(crate) fn run(arguments: ProviderProposeArgs) -> CliResult {
    let loaded = crate::canonical::load_local(&arguments.project)?;
    let request_file = crate::fs::canonical_file(&arguments.request, "provider request")?;
    let response_file = crate::fs::canonical_file(&arguments.response, "provider response")?;
    let context_file = crate::fs::canonical_file(&arguments.context, "proposal context")?;
    let request: ProviderRequestEnvelope =
        super::workflow_io::read_json(&request_file, "provider request")?;
    let response: ProviderResponseEnvelope =
        super::workflow_io::read_json(&response_file, "provider response")?;
    let context: CaptionProposalContext =
        super::workflow_io::read_json(&context_file, "proposal context")?;
    let proposal = super::workflow_io::result(
        propose_edit(&loaded.envelope, &request, &response, &context),
        "PROVIDER_PROPOSAL_FAILED",
    )?;
    let bytes = super::workflow_io::result(
        canonical_edit_proposal_bytes(&proposal),
        "PROVIDER_PROPOSAL_FAILED",
    )?;
    let mut protected = vec![
        loaded.project_file.clone(),
        request_file,
        response_file,
        context_file,
    ];
    protected.extend(crate::canonical::local_material_paths(
        &loaded.envelope,
        &loaded.project_file,
    ));
    super::workflow_io::write(&bytes, arguments.output.as_deref(), &protected)
}
