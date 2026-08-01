use crate::authoring::*;

#[test]
fn delivery_recipe_defaults_preserve_the_public_authoring_contract() {
    let frames = ImageSequenceEncoding::default();
    assert_eq!(frames.format, ImageFormat::Png);
    assert_eq!(frames.start_number, 0);

    let captions = CaptionSidecarEncoding::default();
    assert_eq!(captions.format, CaptionSidecarFormat::WebVtt);
    assert!(captions.track_ids.is_empty());

    let stem = AudioStemEncoding::default();
    assert_eq!(stem.format, AudioStemFormat::Wav);
    assert_eq!(stem.audio.codec, AudioCodec::PcmS24Le);
    assert_eq!(stem.audio.sample_rate, 48_000);
    assert_eq!(stem.audio.channels, 2);
    assert_eq!(stem.source, AudioMixSourceDecl::Master);

    let scope = ScopeEncoding::default();
    assert_eq!(scope.scope, VideoScope::Waveform);
    assert_eq!(scope.at.raw, "0s");
    assert_eq!(scope.at.span, Span::default());
    assert_eq!((scope.width, scope.height), (1920, 1080));
    assert_eq!(scope.format, ImageFormat::Png);
}

#[test]
fn delivery_audio_layouts_expose_their_exact_channel_counts() {
    assert_eq!(AudioChannelLayoutDecl::Mono.count(), 1);
    assert_eq!(AudioChannelLayoutDecl::Stereo.count(), 2);
}
