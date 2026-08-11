use super::*;
use veac_artifact::{
    ArtifactParameters, OpticalFlowMethod, OpticalFlowSpec, ProxyAudioSpec, ProxyVideoSpec,
    SourceClockSpec, SourceSegmentAudioSpec, SourceSegmentSpec, ThumbnailSpec, WaveformSpec,
};
use veac_ir::{Rational, RationalTime, StreamSelection};

#[test]
fn every_authored_parameter_reaches_the_descriptor_unchanged() {
    let actual = operations()
        .iter()
        .map(convert)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let expected = expected_specs();
    assert_eq!(actual, expected);

    for spec in expected {
        let request = veac_artifact::MediaArtifactRequest {
            source_identity: veac_artifact::ContentDigest::sha256(b"source"),
            producer: veac_artifact::ProducerFingerprint {
                name: "test".into(),
                version: "1".into(),
                configuration: veac_artifact::ContentDigest::sha256(b"configuration"),
            },
            spec: spec.clone(),
        };
        assert_eq!(request.descriptor().unwrap().parameters, parameters(spec));
    }
}

fn expected_specs() -> Vec<MediaArtifactSpec> {
    vec![
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream(0),
            source_clock: identity(2, 5),
            width: 16,
            height: 16,
            frame_rate: rate(10, 1),
            crf: 28,
        }),
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: stream(1),
            source_clock: identity(2, 5),
            sample_rate: 48_000,
            channels: 2,
        }),
        MediaArtifactSpec::Thumbnail(ThumbnailSpec {
            source_stream: stream(0),
            at: time(1, 10),
            width: 16,
            height: 16,
        }),
        MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: stream(1),
            source_clock: identity(2, 5),
            sample_rate: 48_000,
            width: 64,
            height: 16,
            color: "#00ff00".into(),
        }),
        MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
            source_stream: stream(0),
            source_clock: identity(2, 5),
            width: 32,
            height: 32,
            frame_rate: rate(20, 1),
            method: OpticalFlowMethod::BlockMatching,
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: stream(0),
            start: time(0, 5),
            duration: time(1, 5),
            width: 16,
            height: 16,
            frame_rate: rate(10, 1),
            audio: Some(SourceSegmentAudioSpec {
                source_stream: stream(1),
                sample_rate: 48_000,
                channels: 2,
            }),
            crf: 23,
        }),
    ]
}

fn parameters(spec: MediaArtifactSpec) -> ArtifactParameters {
    match spec {
        MediaArtifactSpec::ProxyVideo(value) => ArtifactParameters::ProxyVideo(value),
        MediaArtifactSpec::ProxyAudio(value) => ArtifactParameters::ProxyAudio(value),
        MediaArtifactSpec::Waveform(value) => ArtifactParameters::Waveform(value),
        MediaArtifactSpec::Thumbnail(value) => ArtifactParameters::Thumbnail(value),
        MediaArtifactSpec::OpticalFlow(value) => ArtifactParameters::OpticalFlow(value),
        MediaArtifactSpec::SourceSegment(value) => ArtifactParameters::SourceSegment(value),
    }
}

fn stream(global_index: u32) -> StreamSelection {
    StreamSelection {
        global_index,
        type_index: 0,
    }
}

fn identity(value: i64, timescale: u32) -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: time(value, timescale),
    }
}

fn time(value: i64, timescale: u32) -> RationalTime {
    RationalTime::new(value, timescale).unwrap()
}

fn rate(numerator: i64, denominator: u32) -> Rational {
    Rational::new(numerator, denominator).unwrap()
}
