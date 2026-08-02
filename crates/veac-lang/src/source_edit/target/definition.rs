use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! closed_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
            Serialize, Deserialize, JsonSchema,
        )]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }
    };
}

closed_enum!(SourcePresetKind {
    TextStyle,
    TextLayout,
    ModifierStack,
    EffectPipeline,
    ColorPipeline,
    AudioProcessors,
    DeliveryProfile,
});

closed_enum!(SourceTextStyleField {
    FontFamily,
    FontResource,
    Size,
    Weight,
    FontStyle,
    Fill,
    Tracking,
    LineHeight,
    BackgroundColor,
    BackgroundPadding,
    OutlineColor,
    OutlineWidth,
    ShadowColor,
    ShadowOpacity,
    ShadowBlur,
    ShadowOffsetX,
    ShadowOffsetY,
});

closed_enum!(SourceTextLayoutField {
    BoxWidth,
    BoxHeight,
    Wrap,
    Overflow,
    HorizontalAlign,
    VerticalAlign,
    WritingMode,
    Orientation,
    PathStartOffset,
    PathReverse,
    PathAlign,
});

closed_enum!(SourceColorField {
    InputPrimaries,
    InputTransfer,
    InputMatrix,
    InputRange,
    WorkingPrimaries,
    WorkingTransfer,
    WorkingMatrix,
    WorkingRange,
    OutputPrimaries,
    OutputTransfer,
    OutputMatrix,
    OutputRange,
    BasicExposure,
    BasicTemperature,
    BasicTint,
    BasicHighlights,
    BasicShadows,
    BasicFade,
});

closed_enum!(SourceAudioProcessorKind {
    Eq,
    HighPass,
    LowPass,
    Compressor,
    Limiter,
    Gate,
    Loudness,
});

closed_enum!(SourceAudioProcessorField {
    Frequency,
    Q,
    Poles,
    Threshold,
    Ratio,
    Attack,
    Release,
    Knee,
    MakeupGain,
    Mix,
    Ceiling,
    Range,
    Integrated,
    TruePeak,
});

closed_enum!(SourceAudioEqBandField { Frequency, Gain, Q });

closed_enum!(SourceDeliveryArtifactKind {
    Video,
    ImageSequence,
    CaptionSidecar,
    AudioStem,
    Scope,
    AudioFile,
    AnimatedImage,
    StillImage,
    AdaptivePackage,
});

closed_enum!(SourceDeliveryField {
    Target,
    SampleFormat,
    SampleRate,
    ChannelLayout,
});
