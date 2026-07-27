use crate::{
    audio_container_compatible, output_file_compatible, video_container_compatible, AudioCodec,
    OutputFormat, VideoCodec,
};

#[test]
fn every_video_codec_container_pair_has_an_explicit_result() {
    let codecs = [
        VideoCodec::H264,
        VideoCodec::H265,
        VideoCodec::Vp9,
        VideoCodec::Av1,
        VideoCodec::ProRes,
    ];
    let cases: &[(OutputFormat, &[VideoCodec])] = &[
        (
            OutputFormat::Mp4,
            &[VideoCodec::H264, VideoCodec::H265, VideoCodec::Av1],
        ),
        (
            OutputFormat::Mov,
            &[VideoCodec::H264, VideoCodec::H265, VideoCodec::ProRes],
        ),
        (OutputFormat::Mkv, &codecs),
        (OutputFormat::Webm, &[VideoCodec::Vp9, VideoCodec::Av1]),
    ];

    for &(format, accepted) in cases {
        for codec in codecs {
            assert_eq!(
                video_container_compatible(format, codec),
                accepted.contains(&codec),
                "unexpected {format:?}/{codec:?} compatibility"
            );
        }
    }
}

#[test]
fn every_audio_codec_container_pair_has_an_explicit_result() {
    let codecs = [
        AudioCodec::Aac,
        AudioCodec::Opus,
        AudioCodec::Flac,
        AudioCodec::PcmS16Le,
    ];
    let cases: &[(OutputFormat, &[AudioCodec])] = &[
        (OutputFormat::Mp4, &[AudioCodec::Aac]),
        (OutputFormat::Mov, &[AudioCodec::Aac, AudioCodec::PcmS16Le]),
        (OutputFormat::Mkv, &codecs),
        (OutputFormat::Webm, &[AudioCodec::Opus]),
    ];

    for &(format, accepted) in cases {
        for codec in codecs {
            assert_eq!(
                audio_container_compatible(format, codec),
                accepted.contains(&codec),
                "unexpected {format:?}/{codec:?} compatibility"
            );
        }
    }
}

#[test]
fn file_compatibility_uses_the_final_extension_case_insensitively() {
    for (file_name, format, expected) in [
        ("DELIVERY.MP4", OutputFormat::Mp4, true),
        ("archive.delivery.MoV", OutputFormat::Mov, true),
        ("delivery.MKV", OutputFormat::Mkv, true),
        ("delivery.WeBm", OutputFormat::Webm, true),
        (".mp4", OutputFormat::Mp4, true),
        ("delivery", OutputFormat::Mp4, false),
        ("delivery.", OutputFormat::Mov, false),
        ("delivery.mov", OutputFormat::Mp4, false),
        ("delivery.mp4.backup", OutputFormat::Mp4, false),
    ] {
        assert_eq!(
            output_file_compatible(file_name, format),
            expected,
            "unexpected {file_name:?}/{format:?} compatibility"
        );
    }
}
