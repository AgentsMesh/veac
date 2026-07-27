use veac_ir::{
    Animatable, Clip, EditOperation, Interpolation, Keyframe, KeyframeId, ProjectEnvelope,
    RationalTime, VisualProperty,
};

use super::media::{self, RequiredStream};
use super::support::{self, BuiltApplication};
use crate::{
    AutoReframeApplication, AutoReframeRequest, AutoReframeResult, ClipTimeBinding, CropSample,
    OperationBinding, ProposalEvidence, ProviderError, ProviderErrorKind, ProviderResult,
};

pub(super) fn reframe(
    project: &ProjectEnvelope,
    request: &AutoReframeRequest,
    result: &AutoReframeResult,
    context: &AutoReframeApplication,
) -> ProviderResult<BuiltApplication> {
    let (track, clip) = media::source_clip(
        project,
        &context.clip_id,
        &request.video,
        RequiredStream::Video,
        true,
    )?;
    if result.crops.is_empty() {
        return support::unsupported("auto-reframe result has no crop samples to apply");
    }
    let mut built = BuiltApplication::new(Vec::new(), Vec::new());
    append(
        project,
        clip,
        context.time,
        &context.keyframe_id_prefix,
        &result.crops,
        &mut built,
    )?;
    built
        .preconditions
        .extend(media::source_preconditions(track, clip));
    Ok(built)
}

pub(super) fn append(
    project: &ProjectEnvelope,
    clip: &Clip,
    binding: ClipTimeBinding,
    prefix: &str,
    samples: &[CropSample],
    built: &mut BuiltApplication,
) -> ProviderResult<()> {
    if samples.is_empty() {
        return Ok(());
    }
    let times = mapped_times(project, clip, binding, samples)?;
    let keyframes = samples
        .iter()
        .zip(times)
        .enumerate()
        .map(|(index, (sample, time))| {
            Ok(Keyframe {
                id: crop_key_id(prefix, index)?,
                time,
                value: sample.rect,
                interpolation: Interpolation::Linear,
            })
        })
        .collect::<ProviderResult<Vec<_>>>()?;
    let operation_index = support::index(built.operations.len())?;
    let operation = EditOperation::SetVisualProperty {
        clip_id: clip.id.clone(),
        property: VisualProperty::Crop(Some(Animatable::Keyframes { keyframes })),
    };
    built.evidence.push(ProposalEvidence::CropSamples {
        operation: OperationBinding::new(operation_index, &operation)?,
        sample_indices: (0..samples.len())
            .map(support::index)
            .collect::<ProviderResult<_>>()?,
        time: binding,
        timebase: project.project.timebase,
        keyframe_id_prefix: prefix.to_owned(),
    });
    built.operations.push(operation);
    Ok(())
}

fn mapped_times(
    project: &ProjectEnvelope,
    clip: &Clip,
    binding: ClipTimeBinding,
    samples: &[CropSample],
) -> ProviderResult<Vec<RationalTime>> {
    let mut times = Vec::with_capacity(samples.len());
    for sample in samples {
        let time = super::super::time::clip_time(sample.time, binding, project.project.timebase)?;
        if time.value < 0
            || time > clip.record_range.duration
            || times.last().is_some_and(|previous| *previous >= time)
        {
            return support::invalid("crop samples map outside or out of order on the clip");
        }
        times.push(time);
    }
    Ok(times)
}

pub(super) fn crop_key_id(prefix: &str, index: usize) -> ProviderResult<KeyframeId> {
    KeyframeId::new(format!("{prefix}_crop_{:04}", index + 1)).map_err(|error| {
        ProviderError::with_source(
            ProviderErrorKind::InvalidContract,
            "crop keyframe ID prefix is invalid",
            error,
        )
    })
}
