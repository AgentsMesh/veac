use veac_ir::{Rational, RationalTime, StreamSelection};

use crate::*;

#[test]
fn every_artifact_parameter_variant_reports_and_validates_its_kind() {
    for (expected, value) in all_parameters() {
        assert_eq!(value.kind(), expected);
        value.validate().unwrap();
        let encoded = serde_json::to_value(&value).unwrap();
        assert_eq!(
            serde_json::from_value::<ArtifactParameters>(encoded).unwrap(),
            value
        );
    }
}

fn all_parameters() -> Vec<(ArtifactKind, ArtifactParameters)> {
    let provider = || ProviderResultParameters::new("main").unwrap();
    let output = |path| RenderOutputParameters::new(0, path);
    vec![
        (
            ArtifactKind::ProxyVideo,
            crate::test_support::descriptor().parameters,
        ),
        (
            ArtifactKind::ProxyAudio,
            ArtifactParameters::ProxyAudio(proxy_audio()),
        ),
        (
            ArtifactKind::Waveform,
            ArtifactParameters::Waveform(waveform()),
        ),
        (
            ArtifactKind::Thumbnail,
            ArtifactParameters::Thumbnail(thumbnail()),
        ),
        (ArtifactKind::Speech, ArtifactParameters::Speech(provider())),
        (
            ArtifactKind::Translation,
            ArtifactParameters::Translation(provider()),
        ),
        (
            ArtifactKind::MotionTrack,
            ArtifactParameters::MotionTrack(provider()),
        ),
        (ArtifactKind::Matte, ArtifactParameters::Matte(provider())),
        (
            ArtifactKind::OpticalFlow,
            ArtifactParameters::OpticalFlow(optical_flow()),
        ),
        (
            ArtifactKind::Analysis,
            ArtifactParameters::Analysis(analysis()),
        ),
        (
            ArtifactKind::SourceSegment,
            ArtifactParameters::SourceSegment(source_segment()),
        ),
        (
            ArtifactKind::RenderSegment,
            ArtifactParameters::RenderSegment(
                crate::artifact_parameter_render_tests::valid_segment(),
            ),
        ),
        (
            ArtifactKind::CaptionSidecar,
            ArtifactParameters::CaptionSidecar(output("captions.vtt")),
        ),
        (
            ArtifactKind::AudioStem,
            ArtifactParameters::AudioStem(ProducedArtifactParameters::Provider(provider())),
        ),
        (
            ArtifactKind::AudioFile,
            ArtifactParameters::AudioFile(output("audio.wav")),
        ),
        (
            ArtifactKind::AnimatedImage,
            ArtifactParameters::AnimatedImage(output("loop.gif")),
        ),
        (
            ArtifactKind::StillImage,
            ArtifactParameters::StillImage(output("still.png")),
        ),
        (
            ArtifactKind::AdaptivePackage,
            ArtifactParameters::AdaptivePackage(output("index.m3u8")),
        ),
        (
            ArtifactKind::VideoMaster,
            ArtifactParameters::VideoMaster(ProducedArtifactParameters::Render(output(
                "master.mov",
            ))),
        ),
        (
            ArtifactKind::ImageSequenceFrame,
            ArtifactParameters::ImageSequenceFrame(output("frames/000001.png")),
        ),
        (
            ArtifactKind::VideoWaveform,
            ArtifactParameters::VideoWaveform(output("waveform.mp4")),
        ),
        (
            ArtifactKind::Vectorscope,
            ArtifactParameters::Vectorscope(output("vectorscope.mp4")),
        ),
        (
            ArtifactKind::Histogram,
            ArtifactParameters::Histogram(output("histogram.mp4")),
        ),
        (
            ArtifactKind::RenderCheckpoint,
            ArtifactParameters::RenderCheckpoint(RenderCheckpointParameters::Output(output(
                "checkpoint.json",
            ))),
        ),
    ]
}

fn stream() -> StreamSelection {
    StreamSelection {
        global_index: 0,
        type_index: 0,
    }
}

fn clock() -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: RationalTime::new(10, 10).unwrap(),
    }
}

fn proxy_audio() -> ProxyAudioSpec {
    ProxyAudioSpec {
        source_stream: stream(),
        source_clock: clock(),
        sample_rate: 48_000,
        channels: 2,
    }
}

fn waveform() -> WaveformSpec {
    WaveformSpec {
        source_stream: stream(),
        source_clock: clock(),
        sample_rate: 48_000,
        width: 1280,
        height: 240,
        color: "#ffffff".to_owned(),
    }
}

fn thumbnail() -> ThumbnailSpec {
    ThumbnailSpec {
        source_stream: stream(),
        at: RationalTime::new(1, 10).unwrap(),
        width: 640,
        height: 360,
    }
}

fn optical_flow() -> OpticalFlowSpec {
    OpticalFlowSpec {
        source_stream: stream(),
        source_clock: clock(),
        width: 1280,
        height: 720,
        frame_rate: Rational::new(30, 1).unwrap(),
        method: OpticalFlowMethod::MotionCompensated,
    }
}

fn analysis() -> AnalysisArtifactParameters {
    AnalysisArtifactParameters {
        descriptor: AnalysisDescriptor::SceneBoundaries(SceneBoundaryAnalysisDescriptor {
            sensitivity_millionths: 500_000,
        }),
        result_digest: ContentDigest::sha256(b"analysis"),
    }
}

fn source_segment() -> SourceSegmentSpec {
    SourceSegmentSpec {
        video_stream: stream(),
        start: RationalTime::zero(10).unwrap(),
        duration: RationalTime::new(10, 10).unwrap(),
        width: 1280,
        height: 720,
        frame_rate: Rational::new(30, 1).unwrap(),
        audio: Some(SourceSegmentAudioSpec {
            source_stream: stream(),
            sample_rate: 48_000,
            channels: 2,
        }),
        crf: 24,
    }
}
