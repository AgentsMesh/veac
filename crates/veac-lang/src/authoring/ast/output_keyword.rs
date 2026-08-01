use super::{
    AlphaMode, AudioCodec, AudioStemFormat, CaptionOutput, CaptionSidecarFormat, GifDither,
    HardwareBackend, ImageFormat, OutputFormat, PassMode, PixelFormat, VideoCodec, VideoProfile,
    VideoScope,
};

pub(crate) trait OutputKeyword: Sized {
    fn parse(value: &str) -> Option<Self>;
    fn token(&self) -> &'static str;
}

macro_rules! output_keywords {
    ($ty:ty, $($token:literal => $variant:path),+ $(,)?) => {
        impl OutputKeyword for $ty {
            fn parse(value: &str) -> Option<Self> {
                match value {
                    $($token => Some($variant),)+
                    _ => None,
                }
            }

            fn token(&self) -> &'static str {
                match self {
                    $($variant => $token,)+
                }
            }
        }
    };
}

pub(crate) use output_keywords;

output_keywords!(OutputFormat,
    "mp4" => OutputFormat::Mp4, "mov" => OutputFormat::Mov,
    "mkv" => OutputFormat::Mkv, "webm" => OutputFormat::Webm,
    "mxf" => OutputFormat::Mxf,
);
output_keywords!(VideoCodec,
    "h264" => VideoCodec::H264, "h265" => VideoCodec::H265,
    "vp9" => VideoCodec::Vp9, "av1" => VideoCodec::Av1,
    "prores" => VideoCodec::ProRes, "dnxhr" => VideoCodec::DnxHr,
);
output_keywords!(PixelFormat,
    "yuv420p" => PixelFormat::Yuv420p,
    "yuv420p10le" => PixelFormat::Yuv420p10le,
    "yuv422p" => PixelFormat::Yuv422p,
    "yuv422p10le" => PixelFormat::Yuv422p10le,
    "yuv444p10le" => PixelFormat::Yuv444p10le,
    "yuva444p10le" => PixelFormat::Yuva444p10le,
);
output_keywords!(AlphaMode,
    "opaque" => AlphaMode::Opaque, "straight" => AlphaMode::Straight,
);
output_keywords!(AudioCodec,
    "aac" => AudioCodec::Aac, "opus" => AudioCodec::Opus,
    "flac" => AudioCodec::Flac, "pcm-s16le" => AudioCodec::PcmS16Le,
    "pcm-s24le" => AudioCodec::PcmS24Le, "pcm-s32le" => AudioCodec::PcmS32Le,
);
output_keywords!(CaptionOutput,
    "burn-in" => CaptionOutput::BurnIn, "discard" => CaptionOutput::Discard,
);
output_keywords!(PassMode,
    "single" => PassMode::Single, "two-pass" => PassMode::TwoPass,
);
output_keywords!(HardwareBackend,
    "videotoolbox" => HardwareBackend::VideoToolbox,
    "nvenc" => HardwareBackend::Nvenc, "qsv" => HardwareBackend::Qsv,
    "vaapi" => HardwareBackend::Vaapi,
);
output_keywords!(ImageFormat,
    "png" => ImageFormat::Png, "jpeg" => ImageFormat::Jpeg,
    "tiff" => ImageFormat::Tiff, "exr" => ImageFormat::Exr,
);
output_keywords!(CaptionSidecarFormat,
    "srt" => CaptionSidecarFormat::Srt,
    "web-vtt" => CaptionSidecarFormat::WebVtt,
    "ass" => CaptionSidecarFormat::Ass,
);
output_keywords!(AudioStemFormat,
    "wav" => AudioStemFormat::Wav, "flac" => AudioStemFormat::Flac,
);
output_keywords!(VideoScope,
    "waveform" => VideoScope::Waveform,
    "vectorscope" => VideoScope::Vectorscope,
    "histogram" => VideoScope::Histogram,
);
output_keywords!(GifDither,
    "bayer" => GifDither::Bayer,
    "floyd-steinberg" => GifDither::FloydSteinberg,
    "sierra2" => GifDither::Sierra2,
    "none" => GifDither::None,
);
output_keywords!(VideoProfile,
    "h264-baseline" => VideoProfile::H264Baseline,
    "h264-main" => VideoProfile::H264Main,
    "h264-high" => VideoProfile::H264High,
    "h264-high10" => VideoProfile::H264High10,
    "h265-main" => VideoProfile::H265Main,
    "h265-main10" => VideoProfile::H265Main10,
    "vp9-profile0" => VideoProfile::Vp9Profile0,
    "vp9-profile2" => VideoProfile::Vp9Profile2,
    "av1-main" => VideoProfile::Av1Main,
    "prores-4444" => VideoProfile::ProRes4444,
    "dnxhr-lb" => VideoProfile::DnxHrLb,
    "dnxhr-sq" => VideoProfile::DnxHrSq,
    "dnxhr-hq" => VideoProfile::DnxHrHq,
    "dnxhr-hqx" => VideoProfile::DnxHrHqx,
    "dnxhr-444" => VideoProfile::DnxHr444,
);
