use veac_artifact::FullRenderSegmentMediaProfile;
use veac_ir::{
    MediaProbeSnapshot, PixelFormat, ProbedStream, Rational, RationalTime, VideoCodec, VideoProfile,
};

use super::{contract, media_error};
use crate::workflow::{media::video_duration_matches, WorkflowResult};

pub(super) fn validate(
    probe: &MediaProbeSnapshot,
    stream: &ProbedStream,
    profile: &FullRenderSegmentMediaProfile,
    expected_duration: RationalTime,
) -> WorkflowResult<()> {
    let info = stream
        .video
        .as_ref()
        .ok_or_else(|| media_error("render-segment video facts are missing"))?;
    let settings = &profile.video().video;
    if stream.codec != codec(settings.codec)
        || stream.audio.is_some()
        || stream.time_base.is_none()
        || stream.disposition.attached_picture
        || stream.disposition.timed_thumbnail
        || (info.width, info.height) != (profile.width(), profile.height())
        || info.frame_rate != Some(profile.frame_rate())
        || !pixel_format_matches(
            settings.codec,
            settings.profile,
            settings.pixel_format,
            &info.pixel_format,
        )
        || info.sample_aspect_ratio != square_pixels()
        || info.rotation_degrees != 0
        || !profile_matches(settings.profile, info.profile.as_deref())
        || !level_matches(settings.level.as_deref(), info.level)
    {
        return Err(media_error(
            "render-segment video does not match its exact delivery profile",
        ));
    }
    zero_origin(stream)?;
    let actual = contract::duration(probe, stream)
        .ok_or_else(|| media_error("render-segment video duration is missing"))?;
    if !video_duration_matches(actual, expected_duration, profile.frame_rate())
        || !probe.container_duration.is_some_and(|value| {
            video_duration_matches(value, expected_duration, profile.frame_rate())
        })
    {
        return Err(media_error(
            "render-segment video duration differs from its exact range",
        ));
    }
    Ok(())
}

fn zero_origin(stream: &ProbedStream) -> WorkflowResult<()> {
    if stream.start_time.is_none_or(|value| value.value != 0) {
        return Err(media_error("render-segment video does not start at zero"));
    }
    Ok(())
}

fn square_pixels() -> Rational {
    Rational {
        numerator: 1,
        denominator: 1,
    }
}

fn codec(value: VideoCodec) -> &'static str {
    match value {
        VideoCodec::H264 => "h264",
        VideoCodec::H265 => "hevc",
        VideoCodec::Vp9 => "vp9",
        VideoCodec::Av1 => "av1",
        VideoCodec::ProRes => "prores",
        VideoCodec::DnxHr => "dnxhd",
    }
}

fn pixel_format(value: PixelFormat) -> &'static str {
    match value {
        PixelFormat::Yuv420p => "yuv420p",
        PixelFormat::Yuv420p10le => "yuv420p10le",
        PixelFormat::Yuv422p => "yuv422p",
        PixelFormat::Yuv422p10le => "yuv422p10le",
        PixelFormat::Yuv444p10le => "yuv444p10le",
        PixelFormat::Yuva444p10le => "yuva444p10le",
    }
}

fn pixel_format_matches(
    codec: VideoCodec,
    profile: Option<VideoProfile>,
    authored: PixelFormat,
    observed: &str,
) -> bool {
    if (codec, profile, authored)
        == (
            VideoCodec::ProRes,
            Some(VideoProfile::ProRes4444),
            PixelFormat::Yuva444p10le,
        )
    {
        return matches!(observed, "yuva444p10le" | "yuva444p12le");
    }
    observed == pixel_format(authored)
}

fn profile_matches(expected: Option<VideoProfile>, actual: Option<&str>) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    let Some(actual) = actual else {
        return false;
    };
    let candidates: &[&str] = match expected {
        VideoProfile::H264Baseline => &["Baseline", "Constrained Baseline", "66", "578"],
        VideoProfile::H264Main => &["Main", "77"],
        VideoProfile::H264High => &["High", "100"],
        VideoProfile::H264High10 => &["High 10", "High 10 Intra", "110", "2158"],
        VideoProfile::H265Main => &["Main", "1"],
        VideoProfile::H265Main10 => &["Main 10", "Main 10 Intra", "2"],
        VideoProfile::Vp9Profile0 => &["Profile 0", "0"],
        VideoProfile::Vp9Profile2 => &["Profile 2", "2"],
        VideoProfile::Av1Main => &["Main", "0"],
        VideoProfile::ProRes4444 => &["4444", "4"],
        VideoProfile::DnxHrLb => &["DNXHR LB", "1"],
        VideoProfile::DnxHrSq => &["DNXHR SQ", "2"],
        VideoProfile::DnxHrHq => &["DNXHR HQ", "3"],
        VideoProfile::DnxHrHqx => &["DNXHR HQX", "4"],
        VideoProfile::DnxHr444 => &["DNXHR 444", "5"],
    };
    candidates
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(actual))
}

fn level_matches(expected: Option<&str>, actual: Option<i32>) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    parse_level(expected).is_some_and(|value| actual == Some(value))
}

fn parse_level(value: &str) -> Option<i32> {
    let mut parts = value.split('.');
    let major = parts.next()?.parse::<i32>().ok()?;
    let minor = parts
        .next()
        .map(str::parse::<i32>)
        .transpose()
        .ok()?
        .unwrap_or(0);
    if parts.next().is_some() || !(0..=9).contains(&minor) {
        return None;
    }
    major.checked_mul(10)?.checked_add(minor)
}

#[cfg(test)]
#[path = "video/tests.rs"]
mod tests;
