use std::cmp::Ordering;
mod ass;
mod failure;
mod format;
mod plain;

use veac_artifact::ExecutionBindings;
use veac_plan::canonical::{CaptionSidecarFormat, CaptionSidecarOutput, Deliverable, RationalTime};
use veac_plan::{ResolvedClip, ResolvedClipSource, ResolvedRenderPlan, ResolvedText};

use super::{
    error::{diagnostic, CodegenErrorKind},
    BackendAction, BackendOutput, BackendPhase, BackendProduct, BackendTask, CodegenErrors,
};

pub(super) struct Cue<'a> {
    pub(super) start: RationalTime,
    pub(super) end: RationalTime,
    pub(super) clip: &'a ResolvedClip,
    pub(super) content: &'a ResolvedText,
    pub(super) speaker: Option<&'a str>,
    pub(super) layer_key: (i32, i32, u32, i64, u32, String),
    pub(super) order: (i32, u32, u32, String, String),
}

pub(super) fn task(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    deliverable: &Deliverable,
    settings: &CaptionSidecarOutput,
) -> Result<BackendTask, CodegenErrors> {
    let path = super::output::bound_path(deliverable, bindings)?;
    let content = render(plan, bindings, settings)?;
    Ok(BackendTask {
        deliverable_id: deliverable.id.clone(),
        phase: BackendPhase::Single,
        product: BackendProduct::CaptionSidecar,
        output: BackendOutput::File(path.clone()),
        action: BackendAction::WriteFile {
            path,
            content: content.into_bytes(),
        },
    })
}

fn render(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    settings: &CaptionSidecarOutput,
) -> Result<String, CodegenErrors> {
    let Some(sequence) = plan
        .sequences
        .iter()
        .find(|value| value.id == plan.entry_sequence_id)
    else {
        return Err(CodegenErrors::one(super::error::missing_entry(plan)));
    };
    let mut cues = Vec::new();
    for track_id in &settings.track_ids {
        let track = sequence
            .tracks
            .iter()
            .find(|track| track.id == *track_id)
            .ok_or_else(|| missing_track(track_id.as_str()))?;
        for clip in &track.clips {
            let ResolvedClipSource::Caption { content, speaker } = &clip.source else {
                continue;
            };
            let end = clip
                .record_range
                .end()
                .map_err(|_| invalid_time(clip.id.as_str()))?;
            if !format::time_exact(settings.format, clip.record_range.start)
                || !format::time_exact(settings.format, end)
            {
                return Err(inexact_time(clip.id.as_str(), settings.format));
            }
            cues.push(Cue {
                start: clip.record_range.start,
                end,
                clip,
                content,
                speaker: speaker.as_deref(),
                layer_key: (
                    track.order,
                    clip.visual
                        .as_ref()
                        .map_or(0, |visual| visual.compositing.z_index),
                    track.source_order,
                    clip.record_range.start.value,
                    clip.source_order,
                    clip.id.to_string(),
                ),
                order: (
                    track.order,
                    track.source_order,
                    clip.source_order,
                    track.id.to_string(),
                    clip.id.to_string(),
                ),
            });
        }
    }
    cues.sort_by(|left, right| {
        left.start
            .partial_cmp(&right.start)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.order.cmp(&right.order))
    });
    format::render(
        settings.format,
        &cues,
        plan.output.width,
        plan.output.height,
        bindings,
    )
    .map_err(sidecar_failure)
}

fn sidecar_failure(value: failure::Failure) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        value.kind,
        value.code,
        value.object_id,
        value.message,
    ))
}

fn missing_track(id: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "CAPTION_TRACK_MISSING",
        Some(id.to_owned()),
        "caption sidecar track is absent from the resolved sequence",
    ))
}

fn invalid_time(id: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "CAPTION_TIME_INVALID",
        Some(id.to_owned()),
        "caption sidecar cue has invalid timing",
    ))
}

fn inexact_time(id: &str, format: CaptionSidecarFormat) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidPlan,
        "CAPTION_TIME_INEXACT",
        Some(id.to_owned()),
        format!(
            "{} cue times must be exactly representable on a {} ms grid",
            match format {
                CaptionSidecarFormat::Ass => "ASS",
                CaptionSidecarFormat::Srt => "SRT",
                CaptionSidecarFormat::WebVtt => "WebVTT",
            },
            if format == CaptionSidecarFormat::Ass {
                10
            } else {
                1
            }
        ),
    ))
}

#[cfg(test)]
mod tests;
