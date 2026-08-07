use crate::authoring::{
    AlphaMode, AnnotationKind, ApplyStageKind, ArtifactKind, ArtifactTargetKind,
    AudioChannelLayoutDecl, AudioCodec, AudioMixSourceKind, AudioProcessorKind, AudioStemFormat,
    CaptionOutput, CaptionSidecarFormat, ColorMatrix, ColorPrimaries, ColorRange, ColorTransfer,
    GeneratorKind, GifDither, GifPlaybackMode, HardwareBackend, HlsAudioCodec, HlsH264ProfileDecl,
    ImageFormat, LayerKind, ModifierKind, MuxLayout, OutputChannelLayout, OutputFormat, PassMode,
    PixelFormat, RelationKindTag, ResourceKind, SourceKind, SyntaxToken, VideoCodec, VideoProfile,
    VideoRateControlKind, VideoScope,
};

pub(super) fn core_kinds() {
    syntax::<ResourceKind>();
    syntax::<LayerKind>();
    syntax::<SourceKind>();
    syntax::<ModifierKind>();
    syntax::<RelationKindTag>();
    syntax::<ArtifactKind>();
    syntax::<GeneratorKind>();
    syntax::<AudioProcessorKind>();
    syntax::<AnnotationKind>();
    syntax::<ApplyStageKind>();
}

pub(super) fn outputs() {
    syntax::<OutputFormat>();
    syntax::<VideoCodec>();
    syntax::<PixelFormat>();
    syntax::<AlphaMode>();
    syntax::<AudioCodec>();
    syntax::<CaptionOutput>();
    syntax::<PassMode>();
    syntax::<HardwareBackend>();
    syntax::<ImageFormat>();
    syntax::<CaptionSidecarFormat>();
    syntax::<AudioStemFormat>();
    syntax::<VideoScope>();
    syntax::<GifDither>();
    syntax::<VideoProfile>();
    syntax::<HlsAudioCodec>();
    syntax::<HlsH264ProfileDecl>();
    syntax::<AudioMixSourceKind>();
    syntax::<ColorPrimaries>();
    syntax::<ColorTransfer>();
    syntax::<ColorMatrix>();
    syntax::<ColorRange>();
    syntax::<MuxLayout>();
    syntax::<OutputChannelLayout>();
    syntax::<AudioChannelLayoutDecl>();
    syntax::<VideoRateControlKind>();
    syntax::<GifPlaybackMode>();
    syntax::<ArtifactTargetKind>();
}

fn syntax<T: SyntaxToken>() {
    assert_eq!(T::ALL.len(), T::TOKENS.len());
    assert_eq!(T::TOKENS.len(), T::ENTRIES.len());
    for ((token, value), canonical) in T::ENTRIES.iter().zip(T::TOKENS) {
        assert_eq!(token, canonical);
        assert_eq!(value.as_str(), *canonical);
        assert!(T::parse(canonical).as_ref() == Some(value));
    }
}
