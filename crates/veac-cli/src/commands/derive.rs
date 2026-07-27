use veac_artifact::{ArtifactStore, MediaArtifactRequest};
use veac_runtime::workflow::MediaWorkflow;

use crate::arguments::DeriveArgs;
use crate::CliResult;

pub(crate) fn run(arguments: DeriveArgs) -> CliResult {
    let input = crate::fs::canonical_file(&arguments.input, "artifact input")?;
    let spec = crate::fs::canonical_file(&arguments.spec, "artifact request")?;
    let request: MediaArtifactRequest = super::workflow_io::read_json_bounded(
        &spec,
        "artifact request",
        veac_artifact::MAX_ARTIFACT_METADATA_BYTES,
    )?;
    let generated = super::workflow_io::workflow(
        MediaWorkflow::with_tools(arguments.ffmpeg, arguments.ffprobe).derive(
            &ArtifactStore::new(arguments.store),
            &input,
            &request,
        ),
        "DERIVE_FAILED",
    )?;
    let bytes = super::workflow_io::canonical(&generated)?;
    super::workflow_io::write(&bytes, None, &[])
}
