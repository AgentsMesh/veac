#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Mp4,
    Mov,
    Mkv,
    Webm,
    Mxf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
    H265,
    Vp9,
    Av1,
    ProRes,
    DnxHr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioCodec {
    Aac,
    Opus,
    PcmS16Le,
    PcmS24Le,
    PcmS32Le,
    Flac,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassMode {
    Single,
    TwoPass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareSelection {
    Auto,
    Software,
    Explicit { backend: HardwareBackend },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareBackend {
    VideoToolbox,
    Nvenc,
    Qsv,
    Vaapi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Tiff,
    Exr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptionSidecarFormat {
    Srt,
    WebVtt,
    Ass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioStemFormat {
    Wav,
    Flac,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoScope {
    Waveform,
    Vectorscope,
    Histogram,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptionOutput {
    BurnIn,
    Discard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Yuv420p,
    Yuv422p,
    Yuv420p10le,
    Yuv422p10le,
    Yuv444p10le,
    Yuva444p10le,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlphaMode {
    Opaque,
    Straight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoProfile {
    H264Baseline,
    H264Main,
    H264High,
    H264High10,
    H265Main,
    H265Main10,
    Vp9Profile0,
    Vp9Profile2,
    Av1Main,
    ProRes4444,
    DnxHrLb,
    DnxHrSq,
    DnxHrHq,
    DnxHrHqx,
    DnxHr444,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorPrimaries {
    Bt709,
    Bt470M,
    Bt470Bg,
    Smpte170M,
    Smpte240M,
    Film,
    Bt2020,
    Smpte428,
    Smpte431,
    Smpte432,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTransfer {
    Bt709,
    Gamma22,
    Gamma28,
    Smpte170M,
    Smpte240M,
    Linear,
    Srgb,
    Bt2020_10,
    Bt2020_12,
    Smpte2084,
    AribStdB67,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMatrix {
    Rgb,
    Bt709,
    Fcc,
    Bt470Bg,
    Smpte170M,
    Smpte240M,
    Ycgco,
    Bt2020Ncl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorRange {
    Limited,
    Full,
}
