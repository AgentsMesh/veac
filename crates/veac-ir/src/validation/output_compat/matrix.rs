use crate::{AudioCodec, OutputFormat, VideoCodec};

pub fn video_container_compatible(format: OutputFormat, codec: VideoCodec) -> bool {
    match format {
        OutputFormat::Mp4 => matches!(codec, VideoCodec::H264 | VideoCodec::H265 | VideoCodec::Av1),
        OutputFormat::Mov => matches!(
            codec,
            VideoCodec::H264 | VideoCodec::H265 | VideoCodec::ProRes
        ),
        OutputFormat::Mkv => codec != VideoCodec::DnxHr,
        OutputFormat::Webm => matches!(codec, VideoCodec::Vp9 | VideoCodec::Av1),
        OutputFormat::Mxf => codec == VideoCodec::DnxHr,
    }
}

pub fn audio_container_compatible(format: OutputFormat, codec: AudioCodec) -> bool {
    match format {
        OutputFormat::Mp4 => codec == AudioCodec::Aac,
        OutputFormat::Mov => matches!(
            codec,
            AudioCodec::Aac | AudioCodec::PcmS16Le | AudioCodec::PcmS24Le | AudioCodec::PcmS32Le
        ),
        OutputFormat::Mkv => true,
        OutputFormat::Webm => codec == AudioCodec::Opus,
        OutputFormat::Mxf => matches!(codec, AudioCodec::PcmS16Le | AudioCodec::PcmS24Le),
    }
}

pub fn output_file_compatible(file_name: &str, format: OutputFormat) -> bool {
    let extension = file_name.rsplit_once('.').map(|(_, value)| value);
    let expected = match format {
        OutputFormat::Mp4 => "mp4",
        OutputFormat::Mov => "mov",
        OutputFormat::Mkv => "mkv",
        OutputFormat::Webm => "webm",
        OutputFormat::Mxf => "mxf",
    };
    extension.is_some_and(|value| value.eq_ignore_ascii_case(expected))
}
