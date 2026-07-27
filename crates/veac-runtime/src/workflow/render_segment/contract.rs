use veac_artifact::{artifact_key, ArtifactRecord, FullRenderSegmentContract};
use veac_ir::{MediaProbeSnapshot, ProbedStream, ProbedStreamType, StreamSelection};

use super::{audio, container, contract_error, media_error, video};
use crate::workflow::WorkflowResult;

pub(super) fn validate_record(
    segment: &FullRenderSegmentContract,
    record: &ArtifactRecord,
    max_payload_bytes: u64,
) -> WorkflowResult<()> {
    segment
        .descriptor()
        .validate()
        .map_err(|error| contract_source("render-segment descriptor is invalid", error))?;
    record
        .key
        .validate()
        .map_err(|error| contract_source("render-segment key is invalid", error))?;
    record
        .content
        .validate()
        .map_err(|error| contract_source("render-segment content identity is invalid", error))?;
    let key = artifact_key(segment.descriptor())
        .map_err(|error| contract_source("render-segment key cannot be derived", error))?;
    if record.key != key {
        return Err(contract_error(
            "render-segment record key differs from its descriptor",
        ));
    }
    if record.size_bytes == 0 || record.size_bytes > max_payload_bytes {
        return Err(contract_error(
            "render-segment record size is outside the payload policy",
        ));
    }
    Ok(())
}

pub(super) fn validate_snapshot(
    segment: &FullRenderSegmentContract,
    probe: &MediaProbeSnapshot,
) -> WorkflowResult<()> {
    if probe.schema_version != crate::asset::PROBE_SCHEMA_VERSION
        || probe.selection_policy != crate::asset::STREAM_SELECTION_POLICY
    {
        return Err(media_error(
            "render-segment probe uses an unsupported normalized contract",
        ));
    }
    let profile = segment.media_profile();
    container::validate_snapshot(probe, profile.video().container)?;
    let expected_streams = 1 + usize::from(profile.video().audio.is_some());
    if probe.streams.len() != expected_streams {
        return Err(media_error(
            "render-segment contains an unexpected media stream set",
        ));
    }
    let video_stream = selected(probe, probe.selected_video_stream, ProbedStreamType::Video)?;
    video::validate(probe, video_stream, profile, segment.range().duration)?;
    match &profile.video().audio {
        Some(settings) => {
            let stream = selected(probe, probe.selected_audio_stream, ProbedStreamType::Audio)?;
            audio::validate(probe, stream, settings, segment.range().duration)
        }
        None if probe.selected_audio_stream.is_none() => Ok(()),
        None => Err(media_error(
            "silent render-segment unexpectedly selects an audio stream",
        )),
    }
}

fn selected(
    probe: &MediaProbeSnapshot,
    selection: Option<StreamSelection>,
    expected_type: ProbedStreamType,
) -> WorkflowResult<&ProbedStream> {
    let selection = selection
        .ok_or_else(|| media_error("render-segment is missing a required selected media stream"))?;
    probe
        .streams
        .iter()
        .find(|stream| {
            stream.global_index == selection.global_index
                && stream.type_index == selection.type_index
                && stream.media_type == expected_type
        })
        .ok_or_else(|| media_error("render-segment stream selection is inconsistent"))
}

fn contract_source(message: &str, error: veac_artifact::ArtifactError) -> super::WorkflowError {
    super::WorkflowError::with_source(super::WorkflowErrorKind::InvalidContract, message, error)
}

pub(super) fn duration(
    probe: &MediaProbeSnapshot,
    stream: &ProbedStream,
) -> Option<veac_ir::RationalTime> {
    stream.duration.or(probe.container_duration)
}
