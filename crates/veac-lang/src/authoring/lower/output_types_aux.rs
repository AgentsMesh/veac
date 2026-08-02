use crate::authoring as authored;
use veac_ir as canonical;

pub(super) fn color_space(value: authored::ColorSpace) -> canonical::ColorSpace {
    canonical::ColorSpace {
        primaries: match value.primaries {
            authored::ColorPrimaries::Bt709 => canonical::ColorPrimaries::Bt709,
            authored::ColorPrimaries::Bt470M => canonical::ColorPrimaries::Bt470M,
            authored::ColorPrimaries::Bt470Bg => canonical::ColorPrimaries::Bt470Bg,
            authored::ColorPrimaries::Smpte170M => canonical::ColorPrimaries::Smpte170M,
            authored::ColorPrimaries::Smpte240M => canonical::ColorPrimaries::Smpte240M,
            authored::ColorPrimaries::Film => canonical::ColorPrimaries::Film,
            authored::ColorPrimaries::Bt2020 => canonical::ColorPrimaries::Bt2020,
            authored::ColorPrimaries::Smpte428 => canonical::ColorPrimaries::Smpte428,
            authored::ColorPrimaries::Smpte431 => canonical::ColorPrimaries::Smpte431,
            authored::ColorPrimaries::Smpte432 => canonical::ColorPrimaries::Smpte432,
        },
        transfer: transfer(value.transfer),
        matrix: matrix(value.matrix),
        range: match value.range {
            authored::ColorRange::Limited => canonical::ColorRange::Limited,
            authored::ColorRange::Full => canonical::ColorRange::Full,
        },
    }
}

fn transfer(value: authored::ColorTransfer) -> canonical::ColorTransfer {
    use authored::ColorTransfer as A;
    use canonical::ColorTransfer as C;
    match value {
        A::Bt709 => C::Bt709,
        A::Gamma22 => C::Gamma22,
        A::Gamma28 => C::Gamma28,
        A::Smpte170M => C::Smpte170M,
        A::Smpte240M => C::Smpte240M,
        A::Linear => C::Linear,
        A::Srgb => C::Srgb,
        A::Bt2020_10 => C::Bt2020_10,
        A::Bt2020_12 => C::Bt2020_12,
        A::Smpte2084 => C::Smpte2084,
        A::AribStdB67 => C::AribStdB67,
    }
}

fn matrix(value: authored::ColorMatrix) -> canonical::ColorMatrix {
    use authored::ColorMatrix as A;
    use canonical::ColorMatrix as C;
    match value {
        A::Rgb => C::Rgb,
        A::Bt709 => C::Bt709,
        A::Fcc => C::Fcc,
        A::Bt470Bg => C::Bt470Bg,
        A::Smpte170M => C::Smpte170M,
        A::Smpte240M => C::Smpte240M,
        A::Ycgco => C::Ycgco,
        A::Bt2020Ncl => C::Bt2020Ncl,
    }
}

pub(super) fn profile(value: authored::VideoProfile) -> canonical::VideoProfile {
    use authored::VideoProfile as A;
    use canonical::VideoProfile as C;
    match value {
        A::H264Baseline => C::H264Baseline,
        A::H264Main => C::H264Main,
        A::H264High => C::H264High,
        A::H264High10 => C::H264High10,
        A::H265Main => C::H265Main,
        A::H265Main10 => C::H265Main10,
        A::Vp9Profile0 => C::Vp9Profile0,
        A::Vp9Profile2 => C::Vp9Profile2,
        A::Av1Main => C::Av1Main,
        A::ProRes4444 => C::ProRes4444,
        A::DnxHrLb => C::DnxHrLb,
        A::DnxHrSq => C::DnxHrSq,
        A::DnxHrHq => C::DnxHrHq,
        A::DnxHrHqx => C::DnxHrHqx,
        A::DnxHr444 => C::DnxHr444,
    }
}

pub(super) fn hardware(value: authored::HardwareSelection) -> canonical::HardwareSelection {
    match value {
        authored::HardwareSelection::Auto => canonical::HardwareSelection::Auto,
        authored::HardwareSelection::Software => canonical::HardwareSelection::Software,
        authored::HardwareSelection::Explicit { backend } => {
            canonical::HardwareSelection::Explicit {
                backend: match backend {
                    authored::HardwareBackend::VideoToolbox => {
                        canonical::HardwareBackend::VideoToolbox
                    }
                    authored::HardwareBackend::Nvenc => canonical::HardwareBackend::Nvenc,
                    authored::HardwareBackend::Qsv => canonical::HardwareBackend::Qsv,
                    authored::HardwareBackend::Vaapi => canonical::HardwareBackend::Vaapi,
                },
            }
        }
    }
}

pub(super) fn image_format(value: authored::ImageFormat) -> canonical::ImageFormat {
    match value {
        authored::ImageFormat::Png => canonical::ImageFormat::Png,
        authored::ImageFormat::Jpeg => canonical::ImageFormat::Jpeg,
        authored::ImageFormat::Tiff => canonical::ImageFormat::Tiff,
        authored::ImageFormat::Exr => canonical::ImageFormat::Exr,
    }
}

pub(super) fn caption_format(
    value: authored::CaptionSidecarFormat,
) -> canonical::CaptionSidecarFormat {
    match value {
        authored::CaptionSidecarFormat::Srt => canonical::CaptionSidecarFormat::Srt,
        authored::CaptionSidecarFormat::WebVtt => canonical::CaptionSidecarFormat::WebVtt,
        authored::CaptionSidecarFormat::Ass => canonical::CaptionSidecarFormat::Ass,
    }
}

pub(super) fn stem_format(value: authored::AudioStemFormat) -> canonical::AudioStemFormat {
    match value {
        authored::AudioStemFormat::Wav => canonical::AudioStemFormat::Wav,
        authored::AudioStemFormat::Flac => canonical::AudioStemFormat::Flac,
    }
}

pub(super) fn scope(value: authored::VideoScope) -> canonical::VideoScope {
    match value {
        authored::VideoScope::Waveform => canonical::VideoScope::Waveform,
        authored::VideoScope::Vectorscope => canonical::VideoScope::Vectorscope,
        authored::VideoScope::Histogram => canonical::VideoScope::Histogram,
    }
}
