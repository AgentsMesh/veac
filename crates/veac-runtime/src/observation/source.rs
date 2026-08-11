use std::time::Instant;

use veac_ir::{MediaProbeSnapshot, StreamChoice, StreamIntent};

use super::ObservationSource;
use crate::asset::SystemFfprobe;
use crate::RuntimeError;

pub(crate) fn probe(
    ffprobe: &SystemFfprobe,
    source: &ObservationSource,
    deadline: Instant,
) -> Result<MediaProbeSnapshot, RuntimeError> {
    verify(source)?;
    let intent = StreamIntent {
        video: source
            .video_stream
            .map_or(StreamChoice::Auto, |global_index| {
                StreamChoice::GlobalIndex { global_index }
            }),
        audio: StreamChoice::Disabled,
    };
    let snapshot = ffprobe
        .probe_with_intent_until(&source.path, intent, deadline)
        .map_err(|error| RuntimeError::new(format!("media observation probe failed: {error}")))?;
    verify_probe_identity(&snapshot, source)?;
    Ok(snapshot)
}

fn verify_probe_identity(
    snapshot: &MediaProbeSnapshot,
    source: &ObservationSource,
) -> Result<(), RuntimeError> {
    if snapshot.observed_identity != source.identity {
        return Err(RuntimeError::new(
            "media observation source identity does not match its binding",
        ));
    }
    Ok(())
}

pub(crate) fn verify(source: &ObservationSource) -> Result<(), RuntimeError> {
    veac_artifact::verify_source(&source.path, Some(&source.identity))
        .map(|_| ())
        .map_err(|error| RuntimeError::new(format!("media observation source changed: {error}")))
}

#[cfg(test)]
#[path = "source/tests.rs"]
mod tests;
