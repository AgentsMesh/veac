use veac_artifact::ArtifactKind;
use veac_ir::{MaterialKind, ProjectEnvelope};

use super::media::{self, RequiredStream};
use super::support::{self, BuiltApplication};
use crate::{
    DenoiseRequest, DenoiseResult, InputArtifact, MediaReplacementApplication, ProviderArtifact,
    ProviderResult, RemovalRequest, RemovalResult,
};

pub(super) fn denoise(
    project: &ProjectEnvelope,
    request: &DenoiseRequest,
    result: &DenoiseResult,
    context: &MediaReplacementApplication,
) -> ProviderResult<BuiltApplication> {
    if !matches!(
        result.audio.kind(),
        ArtifactKind::AudioStem | ArtifactKind::ProxyAudio
    ) {
        return support::invalid("denoise replacement requires an audio artifact");
    }
    replace(
        project,
        &request.audio,
        &result.audio,
        context,
        MaterialKind::Audio,
        RequiredStream::Audio,
    )
}

pub(super) fn removal(
    project: &ProjectEnvelope,
    request: &RemovalRequest,
    result: &RemovalResult,
    context: &MediaReplacementApplication,
) -> ProviderResult<BuiltApplication> {
    if !matches!(
        result.video.kind(),
        ArtifactKind::VideoMaster | ArtifactKind::ProxyVideo
    ) {
        return support::invalid("removal replacement requires a video artifact");
    }
    replace(
        project,
        &request.video,
        &result.video,
        context,
        MaterialKind::Video,
        RequiredStream::Video,
    )
}

fn replace(
    project: &ProjectEnvelope,
    input: &InputArtifact,
    artifact: &ProviderArtifact,
    context: &MediaReplacementApplication,
    material_kind: MaterialKind,
    stream: RequiredStream,
) -> ProviderResult<BuiltApplication> {
    let (track, clip) = media::source_clip(project, &context.clip_id, input, stream, true)?;
    let source_start = media::normalized_time(project, context.source_start)?;
    media::output_material(
        project,
        artifact,
        &context.material,
        material_kind,
        stream,
        source_start,
        clip.record_range.duration,
    )?;
    let mut built = BuiltApplication::new(Vec::new(), Vec::new());
    media::append_material(&mut built, &context.material, artifact)?;
    media::append_replacement(
        &mut built,
        &context.clip_id,
        &context.material,
        artifact,
        source_start,
    )?;
    built.preconditions = media::source_preconditions(track, clip);
    Ok(built)
}
