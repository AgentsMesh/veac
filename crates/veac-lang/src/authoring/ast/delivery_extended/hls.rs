#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HlsVideoCodec {
    H264,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HlsAudioCodec {
    Aac,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HlsH264ProfileDecl {
    Baseline,
    Main,
    High,
}
