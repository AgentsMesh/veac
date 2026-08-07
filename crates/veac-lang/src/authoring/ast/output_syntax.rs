mod color;
mod delivery;

use super::{
    AlphaMode, AudioCodec, AudioStemFormat, CaptionOutput, CaptionSidecarFormat, GifDither,
    HardwareBackend, ImageFormat, OutputFormat, PassMode, PixelFormat, VideoCodec, VideoProfile,
    VideoScope,
};

crate::impl_local_syntax_tokens!(OutputFormat,
    OutputFormat::Mp4 => "mp4", OutputFormat::Mov => "mov", OutputFormat::Mkv => "mkv",
    OutputFormat::Webm => "webm", OutputFormat::Mxf => "mxf",
);
crate::impl_local_syntax_tokens!(VideoCodec,
    VideoCodec::H264 => "h264", VideoCodec::H265 => "h265", VideoCodec::Vp9 => "vp9",
    VideoCodec::Av1 => "av1", VideoCodec::ProRes => "prores", VideoCodec::DnxHr => "dnxhr",
);
crate::impl_local_syntax_tokens!(PixelFormat,
    PixelFormat::Yuv420p => "yuv420p", PixelFormat::Yuv420p10le => "yuv420p10le",
    PixelFormat::Yuv422p => "yuv422p", PixelFormat::Yuv422p10le => "yuv422p10le",
    PixelFormat::Yuv444p10le => "yuv444p10le", PixelFormat::Yuva444p10le => "yuva444p10le",
);
crate::impl_local_syntax_tokens!(AlphaMode,
    AlphaMode::Opaque => "opaque", AlphaMode::Straight => "straight",
);
crate::impl_local_syntax_tokens!(AudioCodec,
    AudioCodec::Aac => "aac", AudioCodec::Opus => "opus", AudioCodec::Flac => "flac",
    AudioCodec::PcmS16Le => "pcm-s16le", AudioCodec::PcmS24Le => "pcm-s24le",
    AudioCodec::PcmS32Le => "pcm-s32le",
);
crate::impl_local_syntax_tokens!(CaptionOutput,
    CaptionOutput::BurnIn => "burn-in", CaptionOutput::Discard => "discard",
);
crate::impl_local_syntax_tokens!(PassMode,
    PassMode::Single => "single", PassMode::TwoPass => "two-pass",
);
crate::impl_local_syntax_tokens!(HardwareBackend,
    HardwareBackend::VideoToolbox => "videotoolbox", HardwareBackend::Nvenc => "nvenc",
    HardwareBackend::Qsv => "qsv", HardwareBackend::Vaapi => "vaapi",
);
crate::impl_local_syntax_tokens!(ImageFormat,
    ImageFormat::Png => "png", ImageFormat::Jpeg => "jpeg", ImageFormat::Tiff => "tiff",
    ImageFormat::Exr => "exr",
);
crate::impl_local_syntax_tokens!(CaptionSidecarFormat,
    CaptionSidecarFormat::Srt => "srt", CaptionSidecarFormat::WebVtt => "web-vtt",
    CaptionSidecarFormat::Ass => "ass",
);
crate::impl_local_syntax_tokens!(AudioStemFormat,
    AudioStemFormat::Wav => "wav", AudioStemFormat::Flac => "flac",
);
crate::impl_local_syntax_tokens!(VideoScope,
    VideoScope::Waveform => "waveform", VideoScope::Vectorscope => "vectorscope",
    VideoScope::Histogram => "histogram",
);
crate::impl_local_syntax_tokens!(GifDither,
    GifDither::Bayer => "bayer", GifDither::FloydSteinberg => "floyd-steinberg",
    GifDither::Sierra2 => "sierra2", GifDither::None => "none",
);
crate::impl_local_syntax_tokens!(VideoProfile,
    VideoProfile::H264Baseline => "h264-baseline", VideoProfile::H264Main => "h264-main",
    VideoProfile::H264High => "h264-high", VideoProfile::H264High10 => "h264-high10",
    VideoProfile::H265Main => "h265-main", VideoProfile::H265Main10 => "h265-main10",
    VideoProfile::Vp9Profile0 => "vp9-profile0", VideoProfile::Vp9Profile2 => "vp9-profile2",
    VideoProfile::Av1Main => "av1-main", VideoProfile::ProRes4444 => "prores-4444",
    VideoProfile::DnxHrLb => "dnxhr-lb", VideoProfile::DnxHrSq => "dnxhr-sq",
    VideoProfile::DnxHrHq => "dnxhr-hq", VideoProfile::DnxHrHqx => "dnxhr-hqx",
    VideoProfile::DnxHr444 => "dnxhr-444",
);
