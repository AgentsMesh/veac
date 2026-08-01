use super::{
    AdaptivePackageRecipe, AnimatedImageRecipe, AudioCodec, AudioFileRecipe, AudioOutput,
    AudioStemFormat, CaptionOutput, CaptionSidecarFormat, HardwareSelection, Identifier,
    ImageFormat, NumberLiteral, OutputFormat, PassMode, Span, Spanned, StillImageRecipe,
    VideoOutput, VideoScope,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryDecl {
    pub id: Identifier,
    pub sequence: Identifier,
    pub raster: Option<DeliveryRasterDecl>,
    pub artifacts: Vec<ArtifactDecl>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryRasterDecl {
    pub width: NumberLiteral,
    pub height: NumberLiteral,
    pub frame_rate: NumberLiteral,
    pub captions: CaptionOutput,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactDecl {
    pub id: Identifier,
    pub target: ArtifactTargetDecl,
    pub recipe: ArtifactRecipe,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtifactTargetDecl {
    File(Spanned<String>),
    ImageSequence(Spanned<String>),
    Package(Spanned<String>),
}

impl ArtifactTargetDecl {
    pub fn value(&self) -> &Spanned<String> {
        match self {
            Self::File(value) | Self::ImageSequence(value) | Self::Package(value) => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtifactRecipe {
    Video(VideoEncoding),
    ImageSequence(ImageSequenceEncoding),
    CaptionSidecar(CaptionSidecarEncoding),
    AudioStem(AudioStemEncoding),
    Scope(ScopeEncoding),
    AudioFile(AudioFileRecipe),
    AnimatedImage(AnimatedImageRecipe),
    StillImage(StillImageRecipe),
    AdaptivePackage(AdaptivePackageRecipe),
}

impl ArtifactRecipe {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Video(_) => "video",
            Self::ImageSequence(_) => "image-sequence",
            Self::CaptionSidecar(_) => "caption-sidecar",
            Self::AudioStem(_) => "audio-stem",
            Self::Scope(_) => "scope",
            Self::AudioFile(_) => "audio-file",
            Self::AnimatedImage(_) => "animated-image",
            Self::StillImage(_) => "still-image",
            Self::AdaptivePackage(_) => "adaptive-package",
        }
    }

    pub fn requires_raster(&self) -> bool {
        matches!(
            self,
            Self::Video(_)
                | Self::ImageSequence(_)
                | Self::Scope(_)
                | Self::AnimatedImage(_)
                | Self::StillImage(_)
                | Self::AdaptivePackage(_)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoEncoding {
    pub container: OutputFormat,
    pub video: VideoOutput,
    pub audio: Option<AudioOutput>,
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
    pub source: AudioMixSourceDecl,
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
            source: AudioMixSourceDecl::Master,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioMixSourceDecl {
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
