use std::collections::BTreeSet;

use veac_plan::canonical::{DeliverableKind, OutputFormat};
use veac_plan::ResolvedRenderPlan;

use super::Check;

mod aux;
mod names;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let output = &plan.output;
    if veac_plan::PlanOutputId::new(output.id.as_str()).is_err()
        || veac_plan::canonical::RenderConfigId::new(output.render_config_id.as_str()).is_err()
        || !veac_plan::canonical::render_geometry_valid(
            output.width,
            output.height,
            output.frame_rate,
        )
    {
        check.push(
            "PLAN_OUTPUT_INVALID",
            Some(output.id.to_string()),
            "output geometry or frame rate exceeds the default untrusted-plan render budget",
        );
    }
    names::validate(check, &output.deliverables);
    if output.sequence_id != plan.entry_sequence_id {
        check.push(
            "PLAN_ENTRY_MISMATCH",
            Some(output.id.to_string()),
            "output and plan entry sequence differ",
        );
    }
    let mut ids = BTreeSet::new();
    for deliverable in &output.deliverables {
        if !ids.insert(&deliverable.id) {
            check.push(
                "PLAN_DELIVERABLE_DUPLICATE",
                Some(deliverable.id.to_string()),
                "deliverable IDs must be unique",
            );
        }
        match &deliverable.kind {
            DeliverableKind::Video(settings) => video(check, plan, deliverable, settings),
            _ => aux::validate(check, plan, deliverable),
        }
    }
}

fn video(
    check: &mut Check,
    plan: &ResolvedRenderPlan,
    deliverable: &veac_plan::canonical::Deliverable,
    settings: &veac_plan::canonical::VideoDeliverable,
) {
    let id = Some(deliverable.id.to_string());
    if !veac_plan::canonical::video_container_compatible(settings.container, settings.video.codec) {
        check.push(
            "PLAN_VIDEO_CODEC_INCOMPATIBLE",
            id.clone(),
            "video codec is incompatible with the output container",
        );
    }
    if !veac_plan::canonical::output_file_compatible(&deliverable.file_name, settings.container) {
        check.push(
            "PLAN_VIDEO_FILE_FORMAT_INVALID",
            id.clone(),
            "video file extension is incompatible with its container",
        );
    }
    if !veac_plan::canonical::video_settings_valid(&settings.video)
        || !veac_plan::canonical::video_delivery_valid(settings)
        || !veac_plan::canonical::pixel_geometry_valid(
            plan.output.width,
            plan.output.height,
            settings.video.pixel_format,
        )
    {
        check.push(
            "PLAN_VIDEO_SETTINGS_INVALID",
            id.clone(),
            "video encoder or delivery settings are invalid",
        );
    }
    if settings
        .audio
        .as_ref()
        .is_some_and(|audio| !veac_plan::canonical::audio_output_valid(audio))
    {
        check.push(
            "PLAN_AUDIO_OUTPUT_INVALID",
            id.clone(),
            "audio output sample rate or channel count is invalid",
        );
    }
    if settings.audio.as_ref().is_some_and(|audio| {
        !veac_plan::canonical::audio_container_compatible(settings.container, audio.codec)
    }) {
        check.push(
            "PLAN_AUDIO_CODEC_INCOMPATIBLE",
            id.clone(),
            "audio codec is incompatible with the output container",
        );
    }
    if settings.container == OutputFormat::Mxf
        && !veac_plan::canonical::mxf_geometry_valid(
            plan.output.width,
            plan.output.height,
            plan.output.frame_rate,
        )
    {
        check.push(
            "PLAN_MXF_OUTPUT_INVALID",
            id.clone(),
            "MXF dimensions or frame rate are outside the supported contract",
        );
    }
    if settings.optimize_for_streaming
        && !matches!(settings.container, OutputFormat::Mp4 | OutputFormat::Mov)
    {
        check.push(
            "PLAN_STREAMING_MODE_INVALID",
            id,
            "streaming optimization requires MP4 or MOV",
        );
    }
}
