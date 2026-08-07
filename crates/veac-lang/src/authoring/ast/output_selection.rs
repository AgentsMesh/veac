use super::{AudioCodec, HardwareBackend, HardwareSelection};

crate::define_syntax_tokens! {
    array
    pub enum OutputSentinel {
        None => "none",
        Auto => "auto",
        Software => "software",
        Automatic => "automatic",
        Source => "source",
    }
}

impl OutputSentinel {
    pub const AUTOMATIC_TOKENS: [&'static str; 1] = [Self::Automatic.as_str()];
    pub const NONE_TOKENS: [&'static str; 1] = [Self::None.as_str()];
    pub const SOURCE_TOKENS: [&'static str; 1] = [Self::Source.as_str()];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSelection {
    None,
    Codec(AudioCodec),
}

crate::impl_composite_syntax_tokens!(AudioSelection,
    AudioSelection::None => AudioSelection::None => "none",
    AudioSelection::Codec(AudioCodec::Aac) => AudioSelection::Codec(AudioCodec::Aac) => "aac",
    AudioSelection::Codec(AudioCodec::Opus) => AudioSelection::Codec(AudioCodec::Opus) => "opus",
    AudioSelection::Codec(AudioCodec::PcmS16Le) => AudioSelection::Codec(AudioCodec::PcmS16Le) => "pcm-s16le",
    AudioSelection::Codec(AudioCodec::PcmS24Le) => AudioSelection::Codec(AudioCodec::PcmS24Le) => "pcm-s24le",
    AudioSelection::Codec(AudioCodec::PcmS32Le) => AudioSelection::Codec(AudioCodec::PcmS32Le) => "pcm-s32le",
    AudioSelection::Codec(AudioCodec::Flac) => AudioSelection::Codec(AudioCodec::Flac) => "flac",
);

crate::impl_composite_syntax_tokens!(HardwareSelection,
    HardwareSelection::Auto => HardwareSelection::Auto => "auto",
    HardwareSelection::Software => HardwareSelection::Software => "software",
    HardwareSelection::Explicit { backend: HardwareBackend::VideoToolbox } => HardwareSelection::Explicit { backend: HardwareBackend::VideoToolbox } => "videotoolbox",
    HardwareSelection::Explicit { backend: HardwareBackend::Nvenc } => HardwareSelection::Explicit { backend: HardwareBackend::Nvenc } => "nvenc",
    HardwareSelection::Explicit { backend: HardwareBackend::Qsv } => HardwareSelection::Explicit { backend: HardwareBackend::Qsv } => "qsv",
    HardwareSelection::Explicit { backend: HardwareBackend::Vaapi } => HardwareSelection::Explicit { backend: HardwareBackend::Vaapi } => "vaapi",
);
