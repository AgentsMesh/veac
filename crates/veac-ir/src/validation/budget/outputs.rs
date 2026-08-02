use crate::*;

use super::{super::Validator, timeline::sequence_duration};

pub(super) fn validate(validator: &mut Validator, project: &Project) {
    for output in &project.render_configs {
        let Some(duration) = project
            .sequences
            .iter()
            .find(|value| value.id == output.sequence_id)
            .and_then(sequence_duration)
        else {
            continue;
        };
        for deliverable in &output.deliverables {
            validate_deliverable(validator, output, deliverable, duration);
        }
    }
}

fn validate_deliverable(
    validator: &mut Validator,
    output: &RenderConfig,
    deliverable: &Deliverable,
    duration: RationalTime,
) {
    match &deliverable.kind {
        DeliverableKind::Video(settings) => {
            video_work(validator, output, deliverable, duration);
            if let Some(audio) = &settings.audio {
                audio_work(
                    validator,
                    deliverable,
                    duration,
                    audio.sample_rate,
                    audio.channels,
                );
            }
        }
        DeliverableKind::ImageSequence(_) => video_work(validator, output, deliverable, duration),
        DeliverableKind::AudioStem(settings) => audio_work(
            validator,
            deliverable,
            duration,
            settings.audio.sample_rate,
            settings.audio.channels,
        ),
        DeliverableKind::AudioFile(settings) => {
            let AudioFileEncoding::Mp3(encoding) = &settings.encoding;
            audio_work(
                validator,
                deliverable,
                duration,
                encoding.sample_rate_hz,
                encoding.channel_layout.count(),
            );
        }
        DeliverableKind::AnimatedImage(_) => video_work(validator, output, deliverable, duration),
        DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) => {
            hls_work(validator, output, deliverable, settings, duration)
        }
        DeliverableKind::CaptionSidecar(_)
        | DeliverableKind::Scope(_)
        | DeliverableKind::StillImage(_) => {}
    }
}

fn hls_work(
    validator: &mut Validator,
    output: &RenderConfig,
    deliverable: &Deliverable,
    settings: &HlsPackage,
    duration: RationalTime,
) {
    let Some(raster) = &output.raster else {
        return;
    };
    let frames = units_for_duration(duration, raster.frame_rate).unwrap_or(u128::MAX);
    let renditions = settings.renditions.len() as u128;
    if frames.saturating_mul(renditions) > MAX_VIDEO_FRAMES_PER_DELIVERABLE {
        push(
            validator,
            "BUDGET_VIDEO_FRAMES",
            deliverable,
            "video frames",
        );
    }
    let pixels = settings.renditions.iter().fold(0u128, |total, rendition| {
        total.saturating_add(
            u128::from(rendition.raster.width) * u128::from(rendition.raster.height),
        )
    });
    if frames.saturating_mul(pixels) > MAX_PIXEL_FRAMES_PER_DELIVERABLE {
        push(
            validator,
            "BUDGET_PIXEL_FRAMES",
            deliverable,
            "pixel-frames",
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
            push(
                validator,
                "BUDGET_AUDIO_SAMPLES",
                deliverable,
                "audio samples",
            );
        }
    }
}

fn video_work(
    validator: &mut Validator,
    output: &RenderConfig,
    deliverable: &Deliverable,
    duration: RationalTime,
) {
    let Some(raster) = &output.raster else {
        return;
    };
    raster_work(
        validator,
        output,
        deliverable,
        duration,
        raster.width,
        raster.height,
    );
}

fn raster_work(
    validator: &mut Validator,
    output: &RenderConfig,
    deliverable: &Deliverable,
    duration: RationalTime,
    width: u32,
    height: u32,
) {
    let Some(raster) = &output.raster else {
        return;
    };
    let frames = units_for_duration(duration, raster.frame_rate).unwrap_or(u128::MAX);
    if frames > MAX_VIDEO_FRAMES_PER_DELIVERABLE {
        push(
            validator,
            "BUDGET_VIDEO_FRAMES",
            deliverable,
            "video frames",
        );
    }
    if pixel_frames(frames, width, height) > MAX_PIXEL_FRAMES_PER_DELIVERABLE {
        push(
            validator,
            "BUDGET_PIXEL_FRAMES",
            deliverable,
            "pixel-frames",
        );
    }
}

fn audio_work(
    validator: &mut Validator,
    deliverable: &Deliverable,
    duration: RationalTime,
    sample_rate: u32,
    channels: u8,
) {
    let samples =
        channel_samples_for_duration(duration, sample_rate, channels).unwrap_or(u128::MAX);
    if samples > MAX_AUDIO_SAMPLES_PER_DELIVERABLE {
        push(
            validator,
            "BUDGET_AUDIO_SAMPLES",
            deliverable,
            "audio samples",
        );
    }
}

fn push(validator: &mut Validator, code: &str, deliverable: &Deliverable, label: &str) {
    validator.push(
        code,
        Some(deliverable.id.to_string()),
        "/project/render_configs",
        format!("deliverable {label} exceed the untrusted render execution budget"),
        Some("reduce duration, output rate, or output dimensions"),
    );
}
