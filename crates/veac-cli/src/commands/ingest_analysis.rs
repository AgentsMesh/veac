use veac_artifact::{AnalysisIngestionRequest, ArtifactStore};
use veac_runtime::workflow::MediaWorkflow;

use crate::arguments::IngestAnalysisArgs;
use crate::CliResult;

pub(crate) fn run(arguments: IngestAnalysisArgs) -> CliResult {
    let input = crate::fs::canonical_file(&arguments.input, "analysis source")?;
    let request = crate::fs::canonical_file(&arguments.request, "analysis request")?;
    let request: AnalysisIngestionRequest = super::workflow_io::read_json_bounded(
        &request,
        "analysis ingestion request",
        veac_artifact::MAX_ANALYSIS_PAYLOAD_BYTES,
    )?;
    let generated = super::workflow_io::workflow(
        MediaWorkflow::new("ffmpeg").ingest_analysis(
            &ArtifactStore::new(arguments.store),
            &input,
            &request,
        ),
        "ANALYSIS_INGEST_FAILED",
    )?;
    let bytes = super::workflow_io::canonical(&generated)?;
    super::workflow_io::write(&bytes, None, &[])
}
