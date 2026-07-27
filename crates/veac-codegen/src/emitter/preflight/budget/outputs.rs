use veac_plan::canonical::*;
use veac_plan::ResolvedRenderPlan;

use super::super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    let Some(duration) = plan
        .sequences
        .iter()
        .find(|sequence| sequence.id == plan.output.sequence_id)
        .map(|sequence| sequence.duration)
    else {
        return;
    };
    for deliverable in &plan.output.deliverables {
        match &deliverable.kind {
            DeliverableKind::Video(settings) => {
                video(check, plan, deliverable, duration);
                if let Some(audio) = &settings.audio {
                    audio_work(
                        check,
                        deliverable,
                        duration,
                        audio.sample_rate,
                        audio.channels,
                    );
                }
            }
            DeliverableKind::ImageSequence(_) => video(check, plan, deliverable, duration),
            DeliverableKind::AudioStem(settings) => audio_work(
                check,
                deliverable,
                duration,
                settings.audio.sample_rate,
                settings.audio.channels,
            ),
            DeliverableKind::CaptionSidecar(_) | DeliverableKind::Scope(_) => {}
        }
    }
}

fn video(
    check: &mut Check,
    plan: &ResolvedRenderPlan,
    deliverable: &Deliverable,
    duration: RationalTime,
) {
    let frames = units_for_duration(duration, plan.output.frame_rate).unwrap_or(u128::MAX);
    if frames > MAX_VIDEO_FRAMES_PER_DELIVERABLE {
        check.push(
            "PLAN_BUDGET_VIDEO_FRAMES",
            Some(deliverable.id.to_string()),
            "deliverable video frames exceed the untrusted execution budget",
        );
    }
    if pixel_frames(frames, plan.output.width, plan.output.height)
        > MAX_PIXEL_FRAMES_PER_DELIVERABLE
    {
        check.push(
            "PLAN_BUDGET_PIXEL_FRAMES",
            Some(deliverable.id.to_string()),
            "deliverable pixel-frames exceed the untrusted execution budget",
        );
    }
}

fn audio_work(
    check: &mut Check,
    deliverable: &Deliverable,
    duration: RationalTime,
    sample_rate: u32,
    channels: u8,
) {
    if channel_samples_for_duration(duration, sample_rate, channels).unwrap_or(u128::MAX)
        > MAX_AUDIO_SAMPLES_PER_DELIVERABLE
    {
        check.push(
            "PLAN_BUDGET_AUDIO_SAMPLES",
            Some(deliverable.id.to_string()),
            "deliverable audio samples exceed the untrusted execution budget",
        );
    }
}
