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
            DeliverableKind::AudioFile(settings) => {
                let AudioFileEncoding::Mp3(encoding) = &settings.encoding;
                audio_work(
                    check,
                    deliverable,
                    duration,
                    encoding.sample_rate_hz,
                    encoding.channel_layout.count(),
                );
            }
            DeliverableKind::AnimatedImage(_) => video(check, plan, deliverable, duration),
            DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) => {
                hls(check, plan, deliverable, settings, duration)
            }
            DeliverableKind::CaptionSidecar(_)
            | DeliverableKind::Scope(_)
            | DeliverableKind::StillImage(_) => {}
        }
    }
}

fn hls(
    check: &mut Check,
    plan: &ResolvedRenderPlan,
    deliverable: &Deliverable,
    settings: &HlsPackage,
    duration: RationalTime,
) {
    let Some(raster) = &plan.output.raster else {
        return;
    };
    let frames = units_for_duration(duration, raster.frame_rate).unwrap_or(u128::MAX);
    let renditions = settings.renditions.len() as u128;
    if frames.saturating_mul(renditions) > MAX_VIDEO_FRAMES_PER_DELIVERABLE {
        check.push(
            "PLAN_BUDGET_VIDEO_FRAMES",
            Some(deliverable.id.to_string()),
            "deliverable video frames exceed the untrusted execution budget",
        );
    }
    let pixels = settings.renditions.iter().fold(0u128, |total, rendition| {
        total.saturating_add(
            u128::from(rendition.raster.width) * u128::from(rendition.raster.height),
        )
    });
    if frames.saturating_mul(pixels) > MAX_PIXEL_FRAMES_PER_DELIVERABLE {
        check.push(
            "PLAN_BUDGET_PIXEL_FRAMES",
            Some(deliverable.id.to_string()),
            "deliverable pixel-frames exceed the untrusted execution budget",
        );
    }
    if let Some(audio) = &settings.audio {
        let HlsAudioEncoding::Aac(encoding) = &audio.encoding;
        let samples = channel_samples_for_duration(
            duration,
            encoding.sample_rate_hz,
            encoding.channel_layout.count(),
        )
        .unwrap_or(u128::MAX)
        .saturating_mul(renditions);
        if samples > MAX_AUDIO_SAMPLES_PER_DELIVERABLE {
            check.push(
                "PLAN_BUDGET_AUDIO_SAMPLES",
                Some(deliverable.id.to_string()),
                "deliverable audio samples exceed the untrusted execution budget",
            );
        }
    }
}

fn video(
    check: &mut Check,
    plan: &ResolvedRenderPlan,
    deliverable: &Deliverable,
    duration: RationalTime,
) {
    let Some(raster) = &plan.output.raster else {
        return;
    };
    let frames = units_for_duration(duration, raster.frame_rate).unwrap_or(u128::MAX);
    if frames > MAX_VIDEO_FRAMES_PER_DELIVERABLE {
        check.push(
            "PLAN_BUDGET_VIDEO_FRAMES",
            Some(deliverable.id.to_string()),
            "deliverable video frames exceed the untrusted execution budget",
        );
    }
    if pixel_frames(frames, raster.width, raster.height) > MAX_PIXEL_FRAMES_PER_DELIVERABLE {
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
