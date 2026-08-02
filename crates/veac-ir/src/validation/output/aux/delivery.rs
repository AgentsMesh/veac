use crate::*;

use super::super::super::Validator;
use super::support::{extension_is, image_extension, sequence_duration, source_exists};

pub(super) fn validate(
    validator: &mut Validator,
    value: &Deliverable,
    project: &Project,
    output: &RenderConfig,
    path: &str,
) {
    match &value.kind {
        DeliverableKind::AudioFile(settings) => {
            audio(validator, value, settings, project, output, path)
        }
        DeliverableKind::AnimatedImage(settings) => animated(validator, value, settings, path),
        DeliverableKind::StillImage(settings) => {
            still(validator, value, settings, project, output, path)
        }
        DeliverableKind::AdaptivePackage(settings) => {
            hls(validator, value, settings, project, output, path)
        }
        _ => unreachable!("delivery extension validator received a legacy kind"),
    }
}

fn audio(
    validator: &mut Validator,
    value: &Deliverable,
    settings: &AudioFile,
    project: &Project,
    output: &RenderConfig,
    path: &str,
) {
    let valid = matches!(
        &settings.encoding,
        AudioFileEncoding::Mp3(encoding) if mp3_encoding_valid(encoding)
    ) && value
        .target
        .file_name()
        .is_some_and(|name| extension_is(name, "mp3"));
    if !valid || !source_exists(project, &output.sequence_id, &settings.source) {
        validator.value_error("OUTPUT_AUDIO_FILE", path, value.id.as_str());
    }
}

fn animated(validator: &mut Validator, value: &Deliverable, settings: &AnimatedImage, path: &str) {
    let AnimatedImage::Gif(settings) = settings;
    let playback_valid = !matches!(settings.playback, GifPlayback::Times { count: 0 | 1 });
    if !playback_valid
        || !value
            .target
            .file_name()
            .is_some_and(|name| extension_is(name, "gif"))
    {
        validator.value_error("OUTPUT_ANIMATED_IMAGE", path, value.id.as_str());
    }
}

fn still(
    validator: &mut Validator,
    value: &Deliverable,
    settings: &StillImage,
    project: &Project,
    output: &RenderConfig,
    path: &str,
) {
    validator.time(
        frame_time(settings.frame),
        project.timebase,
        false,
        "OUTPUT_STILL_TIME",
        path,
        value.id.as_str(),
    );
    let duration = sequence_duration(project, &output.sequence_id);
    if duration.is_none_or(|end| frame_time(settings.frame) >= end)
        || !value
            .target
            .file_name()
            .is_some_and(|name| image_extension(name, settings.encoding))
    {
        validator.value_error("OUTPUT_STILL_IMAGE", path, value.id.as_str());
    }
}

fn frame_time(value: FrameSelection) -> RationalTime {
    match value {
        FrameSelection::Containing { at } => at,
    }
}

fn hls(
    validator: &mut Validator,
    value: &Deliverable,
    settings: &AdaptivePackage,
    project: &Project,
    output: &RenderConfig,
    path: &str,
) {
    let AdaptivePackage::Hls(settings) = settings;
    let segment_valid = settings.segment_duration.is_valid()
        && settings.segment_duration.timescale == project.timebase
        && settings.segment_duration.value >= i64::from(project.timebase)
        && settings.segment_duration.value <= i64::from(project.timebase) * 60;
    let renditions_valid = (1..=8).contains(&settings.renditions.len())
        && settings.renditions.iter().all(rendition_valid)
        && ordered_unique_ids(&settings.renditions)
        && unique_rasters(&settings.renditions);
    let audio_valid = settings.audio.as_ref().is_none_or(|audio| {
        matches!(
            &audio.encoding,
            HlsAudioEncoding::Aac(encoding) if hls_aac_encoding_valid(encoding)
        ) && source_exists(project, &output.sequence_id, &audio.source)
    });
    if !segment_valid || !renditions_valid || !audio_valid || value.target.package_name().is_none()
    {
        validator.value_error("OUTPUT_HLS_PACKAGE", path, value.id.as_str());
    }
}

fn rendition_valid(value: &HlsRendition) -> bool {
    let HlsVideoEncoding::H264(encoding) = &value.encoding;
    let video = VideoOutput {
        codec: VideoCodec::H264,
        pixel_format: PixelFormat::Yuv420p,
        alpha: AlphaMode::Opaque,
        color_space: encoding.color_space,
        rate_control: VideoRateControl::Bitrate {
            target_bps: encoding.rate_control.target_bps,
            max_bps: Some(encoding.rate_control.max_bps),
            buffer_size_bits: Some(encoding.rate_control.buffer_size_bits),
        },
        gop_size: None,
        b_frames: encoding.b_frames,
        profile: encoding.profile.map(hls_profile),
        level: encoding.level.clone(),
    };
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

fn hls_profile(value: HlsH264Profile) -> VideoProfile {
    match value {
        HlsH264Profile::Baseline => VideoProfile::H264Baseline,
        HlsH264Profile::Main => VideoProfile::H264Main,
        HlsH264Profile::High => VideoProfile::H264High,
    }
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
