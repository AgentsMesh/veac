use std::path::Path;
use std::time::Instant;

use veac_artifact::{
    MediaArtifactLimits, MediaArtifactRequest, MediaArtifactSpec, SourceClockSpec,
};
use veac_ir::{
    HashAlgorithm, MediaProbeSnapshot, ProbedStream, ProbedStreamType, RationalTime,
    StreamSelection,
};

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};
use crate::asset::{ProbeError, SystemFfprobe};

mod intent;
mod timing;
mod work;

pub(super) fn validate(
    tool: &SystemFfprobe,
    path: &Path,
    request: &MediaArtifactRequest,
    limits: MediaArtifactLimits,
    deadline: Instant,
) -> WorkflowResult<()> {
    let snapshot = tool
        .probe_with_intent_until(path, intent::for_spec(&request.spec), deadline)
        .map_err(probe_error)?;
    if snapshot.observed_identity.algorithm != HashAlgorithm::Sha256
        || snapshot.observed_identity.digest != request.source_identity.value
    {
        return invalid("media preflight observed a different source identity");
    }
    match &request.spec {
        MediaArtifactSpec::ProxyVideo(value) => {
            video_clock(&snapshot, value.source_stream, value.source_clock, limits)
        }
        MediaArtifactSpec::ProxyAudio(value) => {
            audio_clock(&snapshot, value.source_stream, value.source_clock, limits)
        }
        MediaArtifactSpec::Waveform(value) => {
            audio_clock(&snapshot, value.source_stream, value.source_clock, limits)
        }
        MediaArtifactSpec::Thumbnail(value) => {
            let stream = selected(&snapshot, value.source_stream, ProbedStreamType::Video)?;
            timing::require_point(&snapshot, stream, value.at)?;
            work::video(stream, value.at, true, limits)
        }
        MediaArtifactSpec::OpticalFlow(value) => {
            video_clock(&snapshot, value.source_stream, value.source_clock, limits)
        }
        MediaArtifactSpec::SourceSegment(value) => {
            let end = range_end(value.start, value.duration)?;
            let video = selected(&snapshot, value.video_stream, ProbedStreamType::Video)?;
            timing::require_range(&snapshot, video, value.start, end)?;
            work::video(video, end, false, limits)?;
            if let Some(audio) = value.audio {
                let stream = selected(&snapshot, audio.source_stream, ProbedStreamType::Audio)?;
                timing::require_range(&snapshot, stream, value.start, end)?;
                work::audio(stream, end, limits)?;
            }
            Ok(())
        }
    }
}

fn video_clock(
    probe: &MediaProbeSnapshot,
    selection: StreamSelection,
    clock: SourceClockSpec,
    limits: MediaArtifactLimits,
) -> WorkflowResult<()> {
    let stream = selected(probe, selection, ProbedStreamType::Video)?;
    let (start, end) = clock_range(clock)?;
    timing::require_range(probe, stream, start, end)?;
    work::video(stream, end, false, limits)
}

fn audio_clock(
    probe: &MediaProbeSnapshot,
    selection: StreamSelection,
    clock: SourceClockSpec,
    limits: MediaArtifactLimits,
) -> WorkflowResult<()> {
    let stream = selected(probe, selection, ProbedStreamType::Audio)?;
    let (start, end) = clock_range(clock)?;
    timing::require_range(probe, stream, start, end)?;
    work::audio(stream, end, limits)
}

fn clock_range(clock: SourceClockSpec) -> WorkflowResult<(RationalTime, RationalTime)> {
    match clock {
        SourceClockSpec::Identity { duration } => Ok((
            RationalTime::zero(duration.timescale).map_err(|_| resource("invalid clock origin"))?,
            duration,
        )),
        SourceClockSpec::Bounded { logical_range } => Ok((
            logical_range.start,
            range_end(logical_range.start, logical_range.duration)?,
        )),
    }
}

fn range_end(start: RationalTime, duration: RationalTime) -> WorkflowResult<RationalTime> {
    start
        .checked_add(duration)
        .map_err(|_| resource("source range overflowed"))
}

fn selected(
    probe: &MediaProbeSnapshot,
    selection: StreamSelection,
    media_type: ProbedStreamType,
) -> WorkflowResult<&ProbedStream> {
    let selected = match media_type {
        ProbedStreamType::Video => probe.selected_video_stream,
        ProbedStreamType::Audio => probe.selected_audio_stream,
        _ => None,
    };
    probe
        .streams
        .iter()
        .find(|stream| {
            selected == Some(selection)
                && stream.global_index == selection.global_index
                && stream.type_index == selection.type_index
                && stream.media_type == media_type
        })
        .ok_or_else(|| {
            WorkflowError::new(
                WorkflowErrorKind::InvalidContract,
                "artifact source stream does not match pinned probe facts",
            )
        })
}

fn probe_error(error: ProbeError) -> WorkflowError {
    let kind = match error {
        ProbeError::ResourceLimit { .. } => WorkflowErrorKind::ResourceLimit,
        ProbeError::ProcessSpawn { .. }
        | ProbeError::ProcessFailed { .. }
        | ProbeError::VersionFailed { .. } => WorkflowErrorKind::ToolFailure,
        ProbeError::IdentityChanged { .. } => WorkflowErrorKind::SourceIdentityMismatch,
        _ => WorkflowErrorKind::InvalidContract,
    };
    WorkflowError::with_source(kind, "media source preflight failed", error)
}

fn invalid<T>(message: &str) -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::InvalidContract,
        message,
    ))
}

fn resource(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::ResourceLimit, message)
}

#[cfg(test)]
#[path = "preflight/test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "preflight/tests.rs"]
mod tests;
