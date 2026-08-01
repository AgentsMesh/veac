use veac_plan::canonical::*;
use veac_plan::{ResolvedAudioRoute, ResolvedRenderPlan, ResolvedSequence};

use super::super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan, value: &Deliverable) {
    let Some(sequence) = plan
        .sequences
        .iter()
        .find(|item| item.id == plan.entry_sequence_id)
    else {
        return;
    };
    let (valid, code, message) = match &value.kind {
        DeliverableKind::AudioFile(settings) => (
            audio(value, settings, sequence),
            "PLAN_AUDIO_FILE_INVALID",
            "audio file encoding, extension, or source is not executable",
        ),
        DeliverableKind::AnimatedImage(AnimatedImage::Gif(settings)) => (
            !matches!(settings.playback, GifPlayback::Times { count: 0 | 1 })
                && value
                    .target
                    .file_name()
                    .is_some_and(|name| super::aux::extension_is(name, "gif")),
            "PLAN_ANIMATED_IMAGE_INVALID",
            "animated-image format or extension is invalid",
        ),
        DeliverableKind::StillImage(settings) => (
            value
                .target
                .file_name()
                .is_some_and(|name| super::aux::image_extension(name, settings.encoding))
                && plan.output.raster.as_ref().is_some_and(|raster| {
                    let FrameSelection::Containing { at } = settings.frame;
                    super::aux::local_snapshot(
                        at,
                        sequence.duration,
                        plan.header.source.timebase,
                        raster.frame_rate,
                    )
                }),
            "PLAN_STILL_IMAGE_INVALID",
            "still-image format or timeline sample is invalid",
        ),
        DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(settings)) => (
            hls(value, settings, sequence, plan.header.source.timebase),
            "PLAN_HLS_PACKAGE_INVALID",
            "HLS package settings are not executable",
        ),
        _ => unreachable!("extended delivery preflight received a legacy kind"),
    };
    if !valid {
        check.push(code, Some(value.id.to_string()), message);
    }
}

fn audio(value: &Deliverable, settings: &AudioFile, sequence: &ResolvedSequence) -> bool {
    matches!(&settings.encoding, AudioFileEncoding::Mp3(encoding) if mp3_encoding_valid(encoding))
        && value
            .target
            .file_name()
            .is_some_and(|name| super::aux::extension_is(name, "mp3"))
        && source(sequence, &settings.source)
}

fn source(sequence: &ResolvedSequence, value: &AudioMixSource) -> bool {
    match value {
        AudioMixSource::Master => true,
        AudioMixSource::Track { track_id } => sequence.tracks.iter().any(|track| {
            track.id == *track_id && matches!(track.kind, TrackKind::Video | TrackKind::Audio)
        }),
        AudioMixSource::Bus { bus_id } => sequence.tracks.iter().any(|track| {
            matches!(track.kind, TrackKind::Video | TrackKind::Audio)
                && matches!(&track.routing.audio, Some(ResolvedAudioRoute::Bus { bus_id: value }) if value == bus_id.as_str())
        }),
    }
}

fn hls(
    value: &Deliverable,
    settings: &HlsPackage,
    sequence: &ResolvedSequence,
    timebase: u32,
) -> bool {
    settings.segment_duration.is_valid()
        && settings.segment_duration.timescale == timebase
        && (i64::from(timebase)..=i64::from(timebase) * 60)
            .contains(&settings.segment_duration.value)
        && (1..=8).contains(&settings.renditions.len())
        && settings.renditions.iter().all(hls_rendition)
        && ordered_unique_ids(&settings.renditions)
        && unique_rasters(&settings.renditions)
        && settings.audio.as_ref().is_none_or(|audio| {
            matches!(&audio.encoding, HlsAudioEncoding::Aac(value) if hls_aac_encoding_valid(value))
                && source(sequence, &audio.source)
        })
        && value.target.package_name().is_some()
}

fn hls_rendition(value: &HlsRendition) -> bool {
    let video = super::super::super::hls_encoding::video(&value.encoding);
    ffmpeg_dimensions_valid(value.raster.width, value.raster.height)
        && pixel_geometry_valid(
            value.raster.width,
            value.raster.height,
            PixelFormat::Yuv420p,
        )
        && video_settings_valid(&video)
        && video_color_delivery_valid(&video)
        && matches!(
            video.rate_control,
            VideoRateControl::Bitrate {
                target_bps: 1..=900_000_000,
                max_bps: Some(_),
                buffer_size_bits: Some(_),
            }
        )
}

fn ordered_unique_ids(values: &[HlsRendition]) -> bool {
    values.windows(2).all(|pair| pair[0].id < pair[1].id)
}

fn unique_rasters(values: &[HlsRendition]) -> bool {
    values.iter().enumerate().all(|(index, value)| {
        values[..index].iter().all(|other| {
            (other.raster.width, other.raster.height) != (value.raster.width, value.raster.height)
        })
    })
}
