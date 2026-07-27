use std::path::Path;
use std::time::Instant;

use veac_artifact::MediaArtifactSpec;
use veac_ir::MediaProbeSnapshot;

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};
use crate::asset::SystemFfprobe;

mod intent;
mod snapshot;
mod timing;

pub(crate) use timing::{audio as audio_duration_matches, video as video_duration_matches};

pub(super) fn validate(
    tool: &SystemFfprobe,
    path: &Path,
    spec: &MediaArtifactSpec,
    deadline: Instant,
) -> WorkflowResult<()> {
    let streams = intent::expected(spec).streams;
    let probe = tool
        .probe_with_intent_until(path, streams, deadline)
        .map_err(|error| {
            let kind = if matches!(error, crate::asset::ProbeError::ResourceLimit { .. }) {
                WorkflowErrorKind::ResourceLimit
            } else {
                WorkflowErrorKind::ToolFailure
            };
            WorkflowError::with_source(kind, "derived artifact cannot be probed", error)
        })?;
    snapshot::validate(&probe, spec)
}

pub(super) fn validate_snapshot(
    probe: &MediaProbeSnapshot,
    spec: &MediaArtifactSpec,
) -> WorkflowResult<()> {
    snapshot::validate(probe, spec)
}

#[cfg(test)]
#[path = "postflight/test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "postflight/tests.rs"]
mod tests;
