use super::{
    AudioCodec, AudioOutput, AudioStemFormat, CaptionOutput, CaptionSidecarFormat,
    HardwareSelection, Identifier, ImageFormat, NumberLiteral, OutputFormat, PassMode, Span,
    Spanned, VideoOutput, VideoScope,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputDecl {
    pub id: Identifier,
    pub sequence: Identifier,
    pub file_name: Spanned<String>,
    pub encoding: OutputEncoding,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputEncoding {
    Video(VideoEncoding),
    ImageSequence(ImageSequenceEncoding),
    CaptionSidecar(CaptionSidecarEncoding),
    AudioStem(AudioStemEncoding),
    Scope(ScopeEncoding),
}

impl OutputEncoding {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Video(_) => "video",
            Self::ImageSequence(_) => "image-sequence",
            Self::CaptionSidecar(_) => "caption-sidecar",
            Self::AudioStem(_) => "audio-stem",
            Self::Scope(_) => "scope",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoEncoding {
    pub container: OutputFormat,
    pub video: VideoOutput,
    pub audio: Option<AudioOutput>,
    pub captions: CaptionOutput,
    pub optimize_for_streaming: bool,
    pub pass_mode: PassMode,
    pub hardware: HardwareSelection,
}

impl Default for VideoEncoding {
    fn default() -> Self {
        Self {
            container: OutputFormat::Mp4,
            video: VideoOutput::default(),
            audio: Some(AudioOutput::default()),
            captions: CaptionOutput::BurnIn,
            optimize_for_streaming: false,
            pass_mode: PassMode::Single,
            hardware: HardwareSelection::Auto,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageSequenceEncoding {
    pub format: ImageFormat,
    pub start_number: u32,
}

impl Default for ImageSequenceEncoding {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            start_number: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptionSidecarEncoding {
    pub format: CaptionSidecarFormat,
    pub track_ids: Vec<Identifier>,
}

impl Default for CaptionSidecarEncoding {
    fn default() -> Self {
        Self {
            format: CaptionSidecarFormat::WebVtt,
            track_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioStemEncoding {
    pub format: AudioStemFormat,
    pub audio: AudioOutput,
    pub source: AudioStemSourceDecl,
}

impl Default for AudioStemEncoding {
    fn default() -> Self {
        Self {
            format: AudioStemFormat::Wav,
            audio: AudioOutput {
                codec: AudioCodec::PcmS24Le,
                sample_rate: 48_000,
                channels: 2,
            },
            source: AudioStemSourceDecl::Master,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioStemSourceDecl {
    Master,
    Track(Identifier),
    Bus(Identifier),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeEncoding {
    pub scope: VideoScope,
    pub at: NumberLiteral,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}

impl Default for ScopeEncoding {
    fn default() -> Self {
        Self {
            scope: VideoScope::Waveform,
            at: NumberLiteral {
                raw: "0s".into(),
                span: Span::default(),
            },
            width: 1920,
            height: 1080,
            format: ImageFormat::Png,
        }
    }
}
