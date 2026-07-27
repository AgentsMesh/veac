use crate::{
    AudioCodec, AudioOutput, AudioStreamInfo, PixelFormat, Rational, RationalTime, VideoStreamInfo,
};

pub const FFMPEG_INT_MAX: u32 = i32::MAX as u32;
/// Default per-axis budget for an untrusted plan, not FFmpeg's syntax limit.
pub const MAX_DIMENSION: u32 = 8_192;
/// Default frame-area budget limits large RGBA/float intermediates to 4096 squared.
pub const MAX_FRAME_PIXELS: u64 = 16_777_216;
/// NLE timelines above 240 fps are outside the supported render budget.
pub const MAX_FRAME_RATE: u32 = 240;
/// FLAC stores sample rate in a 20-bit field with 655_350 as its defined maximum.
pub const FLAC_MAX_SAMPLE_RATE: u32 = 655_350;
/// Keep uncompressed delivery within the established high-resolution audio domain.
pub const PCM_MAX_SAMPLE_RATE: u32 = 384_000;
pub const MAX_INPUT_AUDIO_SAMPLE_RATE: u32 = PCM_MAX_SAMPLE_RATE;
pub const MAX_INPUT_AUDIO_CHANNELS: u8 = 64;
pub const MAX_CHANNEL_LAYOUT_BYTES: usize = 128;

pub fn render_geometry_valid(width: u32, height: u32, frame_rate: Rational) -> bool {
    ffmpeg_dimensions_valid(width, height)
        && frame_rate.is_positive()
        && frame_rate.numerator <= i64::from(i32::MAX)
        && frame_rate.denominator <= FFMPEG_INT_MAX
        && i128::from(frame_rate.numerator)
            <= i128::from(MAX_FRAME_RATE) * i128::from(frame_rate.denominator)
}

pub fn pixel_geometry_valid(width: u32, height: u32, format: PixelFormat) -> bool {
    ffmpeg_dimensions_valid(width, height)
        && match format {
            PixelFormat::Yuv420p | PixelFormat::Yuv420p10le => width % 2 == 0 && height % 2 == 0,
            PixelFormat::Yuv422p | PixelFormat::Yuv422p10le => width % 2 == 0,
            PixelFormat::Yuv444p10le | PixelFormat::Yuva444p10le => true,
        }
}

pub fn image_sequence_start_valid(value: u32) -> bool {
    value <= FFMPEG_INT_MAX
}

pub fn image_sequence_range_valid(
    start_number: u32,
    duration: RationalTime,
    frame_rate: Rational,
) -> bool {
    if !image_sequence_start_valid(start_number)
        || !duration.is_valid()
        || duration.value <= 0
        || !frame_rate.is_positive()
    {
        return false;
    }
    let Some(numerator) = i128::from(duration.value).checked_mul(i128::from(frame_rate.numerator))
    else {
        return false;
    };
    let denominator = i128::from(duration.timescale) * i128::from(frame_rate.denominator);
    let Some(rounded) = numerator.checked_add(denominator - 1) else {
        return false;
    };
    let frame_count = rounded / denominator;
    i128::from(start_number) + frame_count - 1 <= i128::from(FFMPEG_INT_MAX)
}

pub fn audio_output_valid(value: &AudioOutput) -> bool {
    (1..=8).contains(&value.channels) && sample_rate_valid(value.codec, value.sample_rate)
}

pub fn ffmpeg_dimensions_valid(width: u32, height: u32) -> bool {
    width > 0
        && height > 0
        && width <= MAX_DIMENSION
        && height <= MAX_DIMENSION
        && u64::from(width) * u64::from(height) <= MAX_FRAME_PIXELS
}

pub fn ffmpeg_sample_rate_valid(value: u32) -> bool {
    value > 0 && value <= FFMPEG_INT_MAX
}

pub fn input_audio_stream_valid(value: &AudioStreamInfo) -> bool {
    (1..=MAX_INPUT_AUDIO_SAMPLE_RATE).contains(&value.sample_rate)
        && (1..=MAX_INPUT_AUDIO_CHANNELS).contains(&value.channels)
        && !value.channel_layout.trim().is_empty()
        && value.channel_layout.len() <= MAX_CHANNEL_LAYOUT_BYTES
        && !value.channel_layout.chars().any(char::is_control)
}

pub fn input_video_geometry_valid(value: &VideoStreamInfo) -> bool {
    ffmpeg_dimensions_valid(value.width, value.height)
        && normalized_video_dimensions(value)
            .is_some_and(|(width, height)| ffmpeg_dimensions_valid(width, height))
}

pub fn normalized_video_dimensions(value: &VideoStreamInfo) -> Option<(u32, u32)> {
    if !value.sample_aspect_ratio.is_positive() {
        return None;
    }
    let width = ceil_ratio(
        u128::from(value.width) * u128::try_from(value.sample_aspect_ratio.numerator).ok()?,
        u128::from(value.sample_aspect_ratio.denominator),
    )?;
    let height = value.height;
    match value.rotation_degrees.rem_euclid(360) {
        0 | 180 => Some((width, height)),
        90 | 270 => Some((height, width)),
        degrees => {
            let radians = f64::from(degrees).to_radians();
            let cosine = radians.cos().abs();
            let sine = radians.sin().abs();
            let rotated_width = (f64::from(width) * cosine + f64::from(height) * sine).ceil();
            let rotated_height = (f64::from(width) * sine + f64::from(height) * cosine).ceil();
            finite_dimension(rotated_width).zip(finite_dimension(rotated_height))
        }
    }
}

fn ceil_ratio(numerator: u128, denominator: u128) -> Option<u32> {
    let rounded = numerator.checked_add(denominator.checked_sub(1)?)? / denominator;
    u32::try_from(rounded).ok()
}

fn finite_dimension(value: f64) -> Option<u32> {
    (value.is_finite() && value >= 1.0 && value <= f64::from(u32::MAX)).then_some(value as u32)
}

fn sample_rate_valid(codec: AudioCodec, sample_rate: u32) -> bool {
    match codec {
        AudioCodec::Aac => matches!(
            sample_rate,
            96_000
                | 88_200
                | 64_000
                | 48_000
                | 44_100
                | 32_000
                | 24_000
                | 22_050
                | 16_000
                | 12_000
                | 11_025
                | 8_000
                | 7_350
        ),
        AudioCodec::Opus => matches!(sample_rate, 48_000 | 24_000 | 16_000 | 12_000 | 8_000),
        AudioCodec::Flac => (1..=FLAC_MAX_SAMPLE_RATE).contains(&sample_rate),
        AudioCodec::PcmS16Le | AudioCodec::PcmS24Le | AudioCodec::PcmS32Le => {
            (1..=PCM_MAX_SAMPLE_RATE).contains(&sample_rate)
        }
    }
}
