use veac_ir::ProjectEnvelope;

use super::media::{self, RequiredStream};
use super::support::BuiltApplication;
use super::visual_media;
use crate::{
    MatteApplication, MatteResult, ProviderArtifact, ProviderResult, SegmentationRequest,
    SegmentationResult,
};

pub(super) fn segmentation(
    project: &ProjectEnvelope,
    request: &SegmentationRequest,
    result: &SegmentationResult,
    context: &MatteApplication,
) -> ProviderResult<BuiltApplication> {
    media::source_clip(
        project,
        &context.target_clip_id,
        &request.video,
        RequiredStream::Video,
        true,
    )?;
    apply(project, &result.matte, context)
}

pub(super) fn matte(
    project: &ProjectEnvelope,
    result: &MatteResult,
    context: &MatteApplication,
) -> ProviderResult<BuiltApplication> {
    apply(project, &result.matte, context)
}

fn apply(
    project: &ProjectEnvelope,
    artifact: &ProviderArtifact,
    context: &MatteApplication,
) -> ProviderResult<BuiltApplication> {
    let (sequence, target_track, target_clip) =
        visual_media::target(project, &context.target_clip_id)?;
    let (matte_track, record_range, source_start) = visual_media::validate_matte(
        project,
        artifact,
        &context.matte,
        sequence,
        target_clip.record_range,
    )?;
    let mut built = BuiltApplication::new(Vec::new(), Vec::new());
    visual_media::append_matte(
        &mut built,
        &context.matte,
        artifact,
        record_range,
        source_start,
    )?;
    visual_media::append_track_matte(
        &mut built,
        &sequence.id,
        &context.target_clip_id,
        &context.matte.clip_id,
        context.mode,
        context.invert,
        artifact,
    )?;
    built
        .preconditions
        .extend(media::source_preconditions(target_track, target_clip));
    media::add_track_precondition(&mut built, matte_track);
    Ok(built)
}
