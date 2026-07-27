use veac_ir::{Rational, RationalTime, StreamSelection, TimeRange};

use crate::{test_support, *};

pub(super) fn video_spec(input: &veac_plan::ResolvedInput) -> MediaArtifactSpec {
    video_spec_for_stream(
        input.video.as_ref().unwrap().selection,
        SourceClockSpec::Bounded {
            logical_range: range(0, 60),
        },
    )
}

pub(super) fn video_spec_for_stream(
    source_stream: StreamSelection,
    source_clock: SourceClockSpec,
) -> MediaArtifactSpec {
    MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
        source_stream,
        source_clock,
        width: 640,
        height: 360,
        frame_rate: Rational::new(30, 1).unwrap(),
        crf: 24,
    })
}

pub(super) fn audio_spec(input: &veac_plan::ResolvedInput) -> MediaArtifactSpec {
    audio_spec_for_stream(input.audio.as_ref().unwrap().selection)
}

pub(super) fn audio_spec_for_stream(source_stream: StreamSelection) -> MediaArtifactSpec {
    MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
        source_stream,
        source_clock: SourceClockSpec::Identity {
            duration: time(600),
        },
        sample_rate: 48_000,
        channels: 2,
    })
}

pub(super) fn proxy_descriptor(
    input: &veac_plan::ResolvedInput,
    role: MediaRole,
    spec: MediaArtifactSpec,
) -> ArtifactDescriptor {
    let kind = match role {
        MediaRole::Video => ArtifactKind::ProxyVideo,
        MediaRole::Audio => ArtifactKind::ProxyAudio,
    };
    descriptor(input, kind, serde_json::to_value(spec).unwrap())
}

pub(super) fn descriptor(
    input: &veac_plan::ResolvedInput,
    kind: ArtifactKind,
    parameters: serde_json::Value,
) -> ArtifactDescriptor {
    ArtifactDescriptor::new(
        kind,
        test_support::producer(),
        vec![ArtifactDependency {
            role: "input".into(),
            identity: source(input),
        }],
        parameters,
    )
}

pub(super) fn source(input: &veac_plan::ResolvedInput) -> ContentDigest {
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: input.observed_identity.digest.clone(),
    }
}

pub(super) fn assert_invalid<T: std::fmt::Debug>(result: ArtifactResult<T>) {
    assert_eq!(result.unwrap_err().kind, ArtifactErrorKind::InvalidContract);
}

pub(super) fn stream(global_index: u32, type_index: u32) -> StreamSelection {
    StreamSelection {
        global_index,
        type_index,
    }
}

pub(super) fn range(start: i64, duration: i64) -> TimeRange {
    TimeRange::new(time(start), time(duration)).unwrap()
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 600).unwrap()
}
