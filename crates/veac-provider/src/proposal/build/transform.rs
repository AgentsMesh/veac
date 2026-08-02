use veac_ir::{
    Animatable, EditOperation, Interpolation, Keyframe, KeyframeId, Precondition, ProjectEnvelope,
    RationalTime, VisualProperty,
};

use super::support::{self, BuiltApplication};
use crate::{
    ProposalEvidence, ProviderError, ProviderErrorKind, ProviderResult, StabilizationRequest,
    StabilizationResult, TrackingResult, TransformApplication, TransformComponent, TransformSample,
};

use super::media::{self, RequiredStream};

pub(super) fn tracking(
    project: &ProjectEnvelope,
    result: &TrackingResult,
    context: &TransformApplication,
) -> ProviderResult<BuiltApplication> {
    if result.transforms.is_empty() {
        return support::unsupported("tracking result has no transform samples to apply");
    }
    curves(project, &result.transforms, context)
}

pub(super) fn stabilization(
    project: &ProjectEnvelope,
    request: &StabilizationRequest,
    result: &StabilizationResult,
    context: &TransformApplication,
) -> ProviderResult<BuiltApplication> {
    let (track, clip) = media::source_clip(
        project,
        &context.clip_id,
        &request.video,
        RequiredStream::Video,
        true,
    )?;
    let mut built = curves(project, &result.transforms, context)?;
    super::crop::append(
        project,
        clip,
        context.time,
        &context.keyframe_id_prefix,
        &result.crops,
        &mut built,
    )?;
    if built.operations.is_empty() {
        return support::unsupported("stabilization result has no canonical properties to apply");
    }
    for precondition in media::source_preconditions(track, clip) {
        if !built.preconditions.contains(&precondition) {
            built.preconditions.push(precondition);
        }
    }
    Ok(built)
}

fn curves(
    project: &ProjectEnvelope,
    samples: &[TransformSample],
    context: &TransformApplication,
) -> ProviderResult<BuiltApplication> {
    let (track, clip) = support::clip(project, &context.clip_id)?;
    if track.state.locked || clip.visual.is_none() {
        return support::invalid("transform target must be an unlocked visual clip");
    }
    let times = mapped_times(project, clip, context, samples)?;
    let mut built = BuiltApplication::new(Vec::new(), Vec::new());
    if !samples.is_empty() {
        push_curve(
            &mut built,
            &context.clip_id,
            VisualProperty::Position(Animatable::Keyframes {
                keyframes: keys(samples, &times, context, "position", |sample| {
                    sample.position
                })?,
            }),
            TransformComponent::Position,
            samples.len(),
        )?;
        push_curve(
            &mut built,
            &context.clip_id,
            VisualProperty::Scale(Animatable::Keyframes {
                keyframes: keys(samples, &times, context, "scale", |sample| sample.scale)?,
            }),
            TransformComponent::Scale,
            samples.len(),
        )?;
        push_curve(
            &mut built,
            &context.clip_id,
            VisualProperty::RotationDegrees(Animatable::Keyframes {
                keyframes: keys(samples, &times, context, "rotation", |sample| {
                    sample.rotation_degrees
                })?,
            }),
            TransformComponent::Rotation,
            samples.len(),
        )?;
    }
    built.preconditions.push(Precondition::TrackUnlocked {
        track_id: track.id.clone(),
    });
    Ok(built)
}

fn mapped_times(
    project: &ProjectEnvelope,
    clip: &veac_ir::Clip,
    context: &TransformApplication,
    samples: &[TransformSample],
) -> ProviderResult<Vec<RationalTime>> {
    let mut times = Vec::with_capacity(samples.len());
    for sample in samples {
        let time =
            super::super::time::clip_time(sample.time, context.time, project.project.timebase)?;
        if time.value < 0
            || time > clip.record_range.duration
            || times.last().is_some_and(|previous| *previous >= time)
        {
            return support::invalid("transform samples map outside or out of order on the clip");
        }
        times.push(time);
    }
    Ok(times)
}

fn keys<T: Clone>(
    samples: &[TransformSample],
    times: &[RationalTime],
    context: &TransformApplication,
    component: &str,
    value: impl Fn(&TransformSample) -> T,
) -> ProviderResult<Vec<Keyframe<T>>> {
    samples
        .iter()
        .zip(times)
        .enumerate()
        .map(|(index, (sample, time))| {
            Ok(Keyframe {
                id: key_id(&context.keyframe_id_prefix, component, index)?,
                time: *time,
                value: value(sample),
                interpolation: Interpolation::Linear,
            })
        })
        .collect()
}

fn key_id(prefix: &str, component: &str, index: usize) -> ProviderResult<KeyframeId> {
    KeyframeId::new(format!("{prefix}_{component}_{:04}", index + 1)).map_err(|error| {
        ProviderError::with_source(
            ProviderErrorKind::InvalidContract,
            "transform keyframe ID prefix is invalid",
            error,
        )
    })
}

fn push_curve(
    built: &mut BuiltApplication,
    clip_id: &veac_ir::ItemId,
    property: VisualProperty,
    component: TransformComponent,
    sample_count: usize,
) -> ProviderResult<()> {
    let operation_index = support::index(built.operations.len())?;
    let operation = EditOperation::SetVisualProperty {
        clip_id: clip_id.clone(),
        property,
    };
    built.evidence.push(ProposalEvidence::TransformSamples {
        operation: crate::OperationBinding::new(operation_index, &operation)?,
        sample_indices: (0..sample_count)
            .map(support::index)
            .collect::<ProviderResult<_>>()?,
        component,
    });
    built.operations.push(operation);
    Ok(())
}
