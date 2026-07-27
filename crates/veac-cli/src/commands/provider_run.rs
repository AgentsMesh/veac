use std::path::PathBuf;

use veac_artifact::ArtifactStore;
use veac_provider::{canonical_response_bytes, ProviderRequestEnvelope};
use veac_runtime::workflow::ProviderRunner;

use crate::arguments::ProviderRunArgs;
use crate::CliResult;

pub(crate) fn run(arguments: ProviderRunArgs) -> CliResult {
    let request_file = crate::fs::canonical_file(&arguments.request, "provider request")?;
    let program = crate::fs::canonical_file(&arguments.program, "provider program")?;
    let request: ProviderRequestEnvelope =
        super::workflow_io::read_json(&request_file, "provider request")?;
    let execution = super::workflow_io::workflow(
        ProviderRunner::new(&program)
            .with_arguments(arguments.arguments)
            .run(&ArtifactStore::new(arguments.store), &request),
        "PROVIDER_RUN_FAILED",
    )?;
    let bytes = super::workflow_io::result(
        canonical_response_bytes(&execution.response),
        "PROVIDER_RESPONSE",
    )?;
    let protected: Vec<PathBuf> = vec![request_file, program];
    super::workflow_io::write(&bytes, arguments.response.as_deref(), &protected)
}
