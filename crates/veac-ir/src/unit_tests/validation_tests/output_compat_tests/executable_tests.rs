use crate::{
    audio_output_valid, ffmpeg_dimensions_valid, ffmpeg_sample_rate_valid,
    image_sequence_range_valid, image_sequence_start_valid, pixel_geometry_valid,
    render_geometry_valid, AudioCodec, AudioOutput, PixelFormat, Rational, RationalTime,
    FFMPEG_INT_MAX, FLAC_MAX_SAMPLE_RATE, MAX_DIMENSION, MAX_FRAME_PIXELS, MAX_FRAME_RATE,
    PCM_MAX_SAMPLE_RATE,
};

#[test]
fn render_geometry_requires_ffmpeg_integer_representability() {
    let rate = Rational::new(60_000, 1_001).unwrap();
    assert!(render_geometry_valid(4_096, 4_096, rate));
    assert_eq!(
        u64::from(4_096_u32) * u64::from(4_096_u32),
        MAX_FRAME_PIXELS
    );
    for (width, height, frame_rate) in [
        (0, 1080, rate),
        (1920, 0, rate),
        (FFMPEG_INT_MAX + 1, 1080, rate),
        (1920, FFMPEG_INT_MAX + 1, rate),
        (MAX_DIMENSION + 1, 1080, rate),
        (1920, MAX_DIMENSION + 1, rate),
        (7_680, 4_320, rate),
        (
            1920,
            1080,
            Rational {
                numerator: i64::from(i32::MAX) + 1,
                denominator: 1,
            },
        ),
        (
            1920,
            1080,
            Rational {
                numerator: 1,
                denominator: FFMPEG_INT_MAX + 1,
            },
        ),
        (
            1920,
            1080,
            Rational::new(i64::from(MAX_FRAME_RATE) + 1, 1).unwrap(),
        ),
    ] {
        assert!(!render_geometry_valid(width, height, frame_rate));
    }
}

#[test]
fn chroma_subsampling_requires_representable_aligned_dimensions() {
    for (format, width, height, expected) in [
        (PixelFormat::Yuv420p, 1920, 1080, true),
        (PixelFormat::Yuv420p10le, 1919, 1080, false),
        (PixelFormat::Yuv420p, 1920, 1079, false),
        (PixelFormat::Yuv422p, 1920, 1079, true),
        (PixelFormat::Yuv422p10le, 1919, 1080, false),
        (PixelFormat::Yuv444p10le, 1919, 1079, true),
        (PixelFormat::Yuva444p10le, 1919, 1079, true),
        (PixelFormat::Yuv444p10le, 0, 1080, false),
    ] {
        assert_eq!(pixel_geometry_valid(width, height, format), expected);
    }
}

#[test]
fn image_start_number_fits_the_ffmpeg_option_domain() {
    assert!(image_sequence_start_valid(FFMPEG_INT_MAX));
    assert!(!image_sequence_start_valid(FFMPEG_INT_MAX + 1));
    assert!(ffmpeg_dimensions_valid(1, MAX_DIMENSION));
    assert!(!ffmpeg_dimensions_valid(1, MAX_DIMENSION + 1));
    assert!(ffmpeg_sample_rate_valid(FFMPEG_INT_MAX));
    assert!(!ffmpeg_sample_rate_valid(FFMPEG_INT_MAX + 1));
}

#[test]
fn image_sequence_final_number_includes_the_rounded_up_frame_count() {
    let one = RationalTime::new(1, 1).unwrap();
    let two = RationalTime::new(2, 1).unwrap();
    let rate = Rational::new(1, 1).unwrap();
    assert!(image_sequence_range_valid(FFMPEG_INT_MAX, one, rate));
    assert!(image_sequence_range_valid(FFMPEG_INT_MAX - 1, two, rate));
    assert!(!image_sequence_range_valid(FFMPEG_INT_MAX, two, rate));
    assert!(!image_sequence_range_valid(
        FFMPEG_INT_MAX,
        RationalTime::new(1_001, 1_000).unwrap(),
        rate,
    ));
    assert!(!image_sequence_range_valid(
        1,
        RationalTime::zero(1).unwrap(),
        rate,
    ));
}

#[test]
fn lossy_audio_sample_rates_match_ffmpeg_eight_encoder_capabilities() {
    let aac = [
        96_000, 88_200, 64_000, 48_000, 44_100, 32_000, 24_000, 22_050, 16_000, 12_000, 11_025,
        8_000, 7_350,
    ];
    let opus = [48_000, 24_000, 16_000, 12_000, 8_000];
    for rate in aac {
        assert!(valid_audio(AudioCodec::Aac, rate, 2));
    }
    for rate in opus {
        assert!(valid_audio(AudioCodec::Opus, rate, 2));
    }
    for rate in [0, 1, 7_351, 48_001, 96_001] {
        assert!(!valid_audio(AudioCodec::Aac, rate, 2));
    }
    for rate in [0, 1, 7_999, 44_100, 48_001] {
        assert!(!valid_audio(AudioCodec::Opus, rate, 2));
    }
}

#[test]
fn lossless_audio_has_bounded_rates_and_one_to_eight_channels() {
    for (codec, maximum) in [
        (AudioCodec::Flac, FLAC_MAX_SAMPLE_RATE),
        (AudioCodec::PcmS16Le, PCM_MAX_SAMPLE_RATE),
        (AudioCodec::PcmS24Le, PCM_MAX_SAMPLE_RATE),
        (AudioCodec::PcmS32Le, PCM_MAX_SAMPLE_RATE),
    ] {
        assert!(valid_audio(codec, 1, 1));
        assert!(valid_audio(codec, maximum, 8));
        assert!(!valid_audio(codec, 0, 2));
        assert!(!valid_audio(codec, maximum + 1, 2));
        assert!(!valid_audio(codec, 48_000, 0));
        assert!(!valid_audio(codec, 48_000, 9));
    }
}

fn valid_audio(codec: AudioCodec, sample_rate: u32, channels: u8) -> bool {
    audio_output_valid(&AudioOutput {
        codec,
        sample_rate,
        channels,
    })
}
