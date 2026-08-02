use veac_ir::{
    HashAlgorithm, MaterialKind, MaterialSource, MediaIdentity, ProbedStream, ProbedStreamType,
    ProjectEnvelope, RationalTime, StreamSelection,
};

use super::{invalid, normalized_time, RequiredStream};
use crate::{MaterialInsertion, ProviderArtifact, ProviderResult};

pub(crate) fn output_material(
    project: &ProjectEnvelope,
    artifact: &ProviderArtifact,
    insertion: &MaterialInsertion,
    expected: MaterialKind,
    stream: RequiredStream,
    source_start: RationalTime,
    duration: RationalTime,
) -> ProviderResult<()> {
    let material = &insertion.material;
    if project
        .project
        .materials
        .iter()
        .any(|value| value.id == material.id)
    {
        return invalid_result("provider material ID already exists");
    }
    if insertion.before_id.is_some() && insertion.after_id.is_some() {
        return invalid_result("provider material accepts one insertion anchor");
    }
    let identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: artifact.record.content.value.clone(),
    };
    if artifact.record.size_bytes == 0
        || material.kind != expected
        || material.identity.as_ref() != Some(&identity)
        || !matches!(&material.source, MaterialSource::File { uri } if portable_uri(uri))
    {
        return invalid_result("provider materialization contract does not match its artifact");
    }
    let probe = material
        .probe
        .as_ref()
        .ok_or_else(|| invalid("provider material requires normalized probe facts"))?;
    let (selection, media_type) = match stream {
        RequiredStream::Audio => (probe.selected_audio_stream, ProbedStreamType::Audio),
        RequiredStream::Video => (probe.selected_video_stream, ProbedStreamType::Video),
    };
    let selected = selected_stream(probe, selection, media_type)?;
    validate_available(project, probe, selected, source_start, duration)
}

fn selected_stream(
    probe: &veac_ir::MediaProbeSnapshot,
    selection: Option<StreamSelection>,
    media_type: ProbedStreamType,
) -> ProviderResult<&ProbedStream> {
    let selection = selection.ok_or_else(|| invalid("provider material has no selected stream"))?;
    probe
        .streams
        .iter()
        .find(|stream| {
            stream.global_index == selection.global_index && stream.media_type == media_type
        })
        .ok_or_else(|| invalid("provider material selected stream is inconsistent"))
}

fn validate_available(
    project: &ProjectEnvelope,
    probe: &veac_ir::MediaProbeSnapshot,
    stream: &ProbedStream,
    source_start: RationalTime,
    duration: RationalTime,
) -> ProviderResult<()> {
    let source_start = normalized_time(project, source_start)?;
    let expected_start = normalized_time(
        project,
        stream
            .start_time
            .unwrap_or(RationalTime::zero(project.project.timebase).unwrap()),
    )?;
    let available = stream
        .duration
        .or(probe.container_duration)
        .ok_or_else(|| invalid("provider material selected stream requires an exact duration"))?;
    if source_start != expected_start || normalized_time(project, available)? < duration {
        invalid_result("provider material cannot cover the requested source range")
    } else {
        Ok(())
    }
}

fn portable_uri(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains(['\\', '\0'])
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && !matches!(segment, "." | ".."))
        && !value
            .split('/')
            .next()
            .is_some_and(|segment| segment.ends_with(':'))
}

fn invalid_result<T>(message: &str) -> ProviderResult<T> {
    Err(invalid(message))
}
